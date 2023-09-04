use crate::db::queries::Queries;
use crate::db::{Address, UserRecord};

impl Queries {
    pub async fn get_user_by_address(&self, address: &Address) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            r#"
               select u.address,
               u.logo_nft,
               u.username,
               u.bio,
               u.twitter,
               u.instagram,
               u.facebook,
               u.link,
               u.email,
               nm.meta -> 'preview' ->> 'source' as avatar_url
                from users u
                         left join nft n on n.address = u.logo_nft and n.owner = u.address
                         left join nft_metadata nm on n.address = nm.nft
                where u.address = $1
            "#,
            address as &Address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_user(
        &self,
        address: Address,
        bio: Option<String>,
        username: Option<String>,
        logo_nft: Option<Address>,
        twitter: Option<String>,
        instagram: Option<String>,
        facebook: Option<String>,
        link: Option<String>,
        email: Option<String>,
    ) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            r#"
                insert into users(address, logo_nft, username, bio, twitter, instagram, facebook, link, email)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                on conflict (address)
                do update set logo_nft  = $2,
                              username  = $3,
                              bio       = $4,
                              twitter   = $5,
                              instagram = $6,
                              facebook  = $7,
                              link      = $8,
                              email     = $9

            "#,
            address as Address,
            logo_nft as Option<Address>,
            username,
            bio,
            twitter,
            instagram,
            facebook,
            link,
            email as Option<String>
        )
        .execute(self.db.as_ref())
        .await;

        Ok(())
    }
}
