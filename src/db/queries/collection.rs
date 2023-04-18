use crate::db::queries::Queries;

use super::*;
use crate::handlers::CollectionListOrder;

use sqlx::{self};

impl Queries {
    pub async fn get_collection(
        &self,
        address: &String,
    ) -> sqlx::Result<Option<NftCollectionDetails>> {
        sqlx::query_as!(
            NftCollectionDetails,
            r#"
                SELECT c.*, 1::bigint as "cnt!", '[]'::json as "previews!"
                FROM nft_collection_details c
                WHERE c.address = $1"#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn collect_collections(&self, ids: &[String]) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(
            NftCollection,
            r#"
                SELECT
                   c.address as "address!",
                   c.owner as "owner!",
                   c.name,
                   c.description,
                   c.updated as "updated!",
                   c.wallpaper,
                   c.logo,
                   c.total_price,
                   c.max_price,
                   c.owners_count,
                   c.verified as "verified!",
                   c.created as "created!",
                   c.first_mint
                , nft.count as "nft_count!",
                count(1) over () as "cnt!"
                FROM nft_collection c
                         LEFT JOIN lateral ( select count(1) as count from nft n where n.collection = c.address) nft on true
                WHERE c.address = ANY($1)"#,
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
                 SELECT
                   c.address as "address!",
                   c.owner as "owner!",
                   c.name,
                   c.description,
                   c.updated as "updated!",
                   c.wallpaper,
                   c.logo,
                   c.total_price,
                   c.max_price,
                   c.owners_count,
                   c.verified as "verified!",
                   c.created as "created!",
                   c.first_mint,
                   nft.count as "nft_count!",
                   count(1) over () as "cnt!"
                FROM nft_collection c
                         LEFT JOIN lateral ( select count(1) as count from nft n where n.collection = c.address) nft on true
                WHERE c.owner = $1
                LIMIT $2 OFFSET $3
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
            None => "c.owners_count DESC".to_string(),
            Some(order) => {
                let field = order.field.to_string();
                format!("c.{field} {}", order.direction)
            }
        };

        let query = format!(
            r#"
                SELECT c.*,
                       count(1) over ()  as "cnt",
                       previews.previews as "previews"
                FROM nft_collection_details c
                         left join lateral (
                    select json_agg(ag2.preview_url) as previews
                    from (select ag.preview_url
                          from (
                                   select nm.meta -> 'preview' as preview_url
                                   from nft n
                                            join nft_metadata nm on n.address = nm.nft
                                       and nm.meta != 'null'
                                   where n.collection = c.address
                                   limit 50) ag
                          order by random()
                          limit 3) ag2
                    ) previews on true
                WHERE (c.owner = ANY ($3) OR array_length($3::varchar[], 1) is null)
                  AND ($4::boolean is false OR c.verified is true)
                  AND ($5::varchar is null OR c.name ILIKE $5)
                  AND (c.address = ANY ($6) OR array_length($6::varchar[], 1) is null)
                ORDER BY {order}
                LIMIT $1 OFFSET $2
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
        sqlx::query_as!(RootRecord, r#"select r.event_whitelist_address as "address!", r.code::text as "code!" from roots r"#)
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
                SELECT c.address        as "address!",
                       c.name,
                       c.description,
                       c.logo,
                       c.verified       as "verified!",
                       count(1) over () as "cnt!",
                       nft.count as "nft_count!"
                FROM nft_collection c
                LEFT JOIN lateral ( select count(1) as count from nft n where n.collection = c.address) nft on true
                where ($3::boolean is false OR c.verified is true)
                AND ($4::varchar is null OR c.name ILIKE $4)
                order by c.owners_count desc
                limit $1 offset $2
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
