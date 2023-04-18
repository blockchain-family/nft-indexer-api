use crate::db::queries::Queries;
use crate::db::{Address, UserRecord};

impl Queries {
    pub async fn get_user_by_address(&self, address: &Address) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            r#"
                select address, logo_nft, username, bio, twitter, instagram, facebook, link, email
                from users where address = $1
            "#,
            address as &Address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }
}
