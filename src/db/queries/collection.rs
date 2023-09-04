use crate::db::queries::Queries;

use super::*;

use sqlx::{self};
use crate::handlers::collection::CollectionListOrder;

impl Queries {
    pub async fn get_collection(
        &self,
        address: &String,
    ) -> sqlx::Result<Option<NftCollectionDetails>> {
        sqlx::query_as!(
            NftCollectionDetails,
            r#"
                select
                c.*,
                1::bigint as "cnt!",
                '[]'::json as "previews!"
                from nft_collection_details c
                where c.address = $1
            "#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn collect_collections(&self, ids: &[String]) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(
            NftCollection,
            r#"
            select
               c.address as "address!",
               c.owner as "owner!",
               c.name,
               c.description,
               c.updated as "updated!",
               c.wallpaper,
               c.logo,
               c.total_price,
               c.max_price,
               nft.owners_count,
               c.verified as "verified!",
               c.created as "created!",
               c.first_mint,
               nft.count as "nft_count!",
               count(1) over () as "cnt!"
            from nft_collection c
            left join lateral (
                select
                    count(1) as count,
                    count(distinct owner)::int as owners_count
                from nft n
                where n.collection = c.address
             ) nft on true
            where c.address = any($1) and owner is not null
            "#,
            ids
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_collections_by_owner(
        &self,
        owner: &String,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(
            NftCollection,
            r#"
            select
              c.address as "address!",
              c.owner as "owner!",
              c.name,
              c.description,
              c.updated as "updated!",
              c.wallpaper,
              c.logo,
              c.total_price,
              c.max_price,
              nft.owners_count,
              c.verified as "verified!",
              c.created as "created!",
              c.first_mint,
              nft.count as "nft_count!",
              count(1) over () as "cnt!"
            from nft_collection c
            left join lateral (
                select
                    count(1) as count,
                    count(distinct owner)::int as owners_count
                from nft n
                where n.collection = c.address
            ) nft on true
            where c.owner = $1
            limit $2
            offset $3
            "#,
            owner,
            limit as i64,
            offset as i64
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_collections(
        &self,
        name: Option<&String>,
        owners: &[String],
        verified: Option<&bool>,
        collections: &[Address],
        limit: usize,
        offset: usize,
        order: Option<CollectionListOrder>,
    ) -> sqlx::Result<Vec<NftCollectionDetails>> {
        let order = match order {
            None => "c.owners_count desc".to_string(),
            Some(order) => {
                let field = order.field.to_string();
                format!("c.{field} {}", order.direction)
            }
        };

        let query = format!(
            r#"
            select
                c.*,
                count(1) over ()  as "cnt",
                previews.previews as "previews"
            from nft_collection_details c
            left join lateral (
                select json_agg(ag2.preview_url) as previews
                from (
                    select ag.preview_url
                    from (
                        select nm.meta -> 'preview' as preview_url
                        from nft n
                        join nft_metadata nm on n.address = nm.nft and
                             nm.meta is not null
                        where n.collection = c.address
                        limit 50
                    ) ag
                    order by random()
                    limit 3
                ) ag2
            ) previews on true
            where (c.owner = any($3) or array_length($3::varchar[], 1) is null) and
                  ($4::boolean is false or c.verified is true) and
                  ($5::varchar is null or c.name ilike $5) and
                  (c.address = any($6) or array_length($6::varchar[], 1) is null)
            order by {order}
            limit $1
            offset $2
            "#
        );

        sqlx::query_as(&query)
            .bind(limit as i64)
            .bind(offset as i64)
            .bind(owners)
            .bind(verified)
            .bind(name)
            .bind(collections)
            .fetch_all(self.db.as_ref())
            .await
    }



    pub async fn list_roots(&self) -> sqlx::Result<Vec<RootRecord>> {
        sqlx::query_as!(
            RootRecord,
            r#"
            select
                r.address as "address!",
                r.code::text as "code!"
            from roots r
            "#
        )
            .fetch_all(self.db.as_ref())
            .await
    }


    pub async fn list_collections_simple(
        &self,
        name: Option<&String>,
        verified: Option<&bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollectionSimple>> {
        sqlx::query_as!(
            NftCollectionSimple,
            r#"
            select
                c.address        as "address!",
                c.name,
                c.description,
                c.logo,
                c.verified       as "verified!",
                count(1) over () as "cnt!",
                nft.count        as "nft_count!"
            from nft_collection c
            left join lateral (
                select
                    count(1) as count,
                    count(distinct owner) as owners_count
                from nft n
                where n.collection = c.address
            ) nft on true
            where ($3::boolean is false or c.verified is true) and
                  ($4::varchar is null or c.name ilike $4)
            order by nft.owners_count desc
            limit $1
            offset $2
            "#,
            limit as i64,
            offset as i64,
            verified,
            name,
        )
            .fetch_all(self.db.as_ref())
            .await
    }

}
