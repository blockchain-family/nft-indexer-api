use chrono::NaiveDateTime;
use crate::db::queries::Queries;
use crate::db::Address;

impl Queries {

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_collection_custom(
        &self,
        address: Address,
        owner: &String,
        updated: NaiveDateTime,
        name: Option<String>,
        description: Option<String>,
        wallpaper: Option<String>,
        logo: Option<String>,
        social: serde_json::Value,
    ) -> sqlx::Result<()> {
         sqlx::query!(
            r#"
                insert into nft_collection_custom(address, owner, updated, name, description, wallpaper, logo, social)
                values ($1, $2, $3, $4, $5, $6, $7, $8)
                on conflict (address)
                do update set updated     = $3,
                              name        = $4,
                              description = $5,
                              wallpaper   = $6,
                              logo        = $7,
                              social      = $8
                where nft_collection_custom.owner = $2
            "#,
            address as _,
            owner as _,
            updated as _,
            name,
            description,
            wallpaper as _,
            logo as _,
            social as serde_json::Value
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }
}
