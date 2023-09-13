use crate::db::queries::Queries;
use crate::db::Address;
use chrono::NaiveDateTime;

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
                insert into nft_collection_custom(address, updated, name, description, wallpaper, logo, social)
                select address, $2, $3, $4, $5, $6, $7 from nft_collection
                where address = $1 and owner = $8
                on conflict (address)
                do update set updated     = $2,
                              name        = $3,
                              description = $4,
                              wallpaper   = $5,
                              logo        = $6,
                              social      = $7
                where nft_collection_custom.address =
                (select nc.address from nft_collection nc where nc.address = $1 and nc.owner = $8)
            "#,
            address as _,
            updated as _,
            name,
            description,
            wallpaper as _,
            logo as _,
            social as serde_json::Value,
            owner as _
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }
}