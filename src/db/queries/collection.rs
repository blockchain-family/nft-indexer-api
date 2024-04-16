use super::*;

use crate::db::{queries::Queries, query_params::collection::CollectionsListParams};

impl Queries {
    pub async fn get_collection(
        &self,
        address: &String,
    ) -> sqlx::Result<Option<NftCollectionDetails>> {
        sqlx::query_as!(
            NftCollectionDetails,
            r#"
            select c.address,
                   c.owner,
                   c.name,
                   c.description,
                   c.created as "created?",
                   c.updated as "updated?",
                   c.verified,
                   c.wallpaper,
                   c.logo,
                   c.owners_count,
                   c.nft_count,
                   c.floor_price_usd,
                   c.total_volume_usd,
                   c.attributes,
                   c.first_mint,
                   c.social,
                   c.royalty,
                   null::numeric as max_price,
                   null::numeric as total_price,
                   1::bigint     as "cnt!",
                   '[]'::json    as "previews!"
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
            select c.address     as "address!",
                   coalesce(c.owner, '0:0000000000000000000000000000000000000000000000000000000000000000')       as "owner!",
                   c.name        as "name",
                   c.description as "description",
                   c.updated     as "updated!",
                   c.wallpaper   as "wallpaper",
                   c.logo        as "logo",
                   null::numeric as total_price,
                   null::numeric as max_price,
                   c.owners_count::int,
                   c.verified    as "verified!",
                   c.created     as "created!",
                   c.first_mint  as "first_mint!",
                   coalesce(c.nft_count,0)   as "nft_count!",
                   c.total_count as "cnt!",
                   c.social      as "social",
                   c.royalty     as "royalty"
            from nft_collection_details c
            where c.address = any ($1)
              --and owner is not null
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
            select c.address     as "address!",
                   c.owner       as "owner!",
                   c.name,
                   c.description,
                   c.updated     as "updated!",
                   c.wallpaper,
                   c.logo,
                   null::numeric as total_price,
                   null::numeric as max_price,
                   c.owners_count::int,
                   c.verified    as "verified!",
                   c.created     as "created!",
                   c.first_mint  as "first_mint!",
                   c.nft_count   as "nft_count!",
                   c.total_count as "cnt!",
                   c.social      as "social",
                   c.royalty     as "royalty"
            from nft_collection_details c
            where c.owner = $1
            limit $2 offset $3
            "#,
            owner,
            limit as i64,
            offset as i64
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_collections(
        &self,
        params: &CollectionsListParams<'_>,
    ) -> sqlx::Result<Vec<NftCollectionDetails>> {
        let order = match &params.order {
            None => "c.owners_count desc".to_string(),
            Some(order) => {
                format!("c.{} {}", order.field, order.direction)
            }
        };

        let query = format!(
            r#"
            select c.address,
                   c.owner,
                   c.name,
                   c.description,
                   c.created,
                   c.updated,
                   c.verified,
                   c.wallpaper,
                   c.logo,
                   c.owners_count,
                   c.nft_count,
                   c.floor_price_usd,
                   c.total_volume_usd,
                   c.attributes,
                   c.first_mint,
                   case when $4::boolean is false then c.total_count else c.verified_count end as "cnt",
                   coalesce(previews.previews, '[]'::json)                                     as "previews",
                   null::numeric                                                               as max_price,
                   null::numeric                                                               as total_price,
                   c.social                                                                    as "social",
                   c.royalty                                                                   as "royalty"
            from nft_collection_details c
                     left join lateral ( select json_agg(ag2.preview_url) as previews
                                         from ( select ag.preview_url
                                                from ( select nm.meta -> 'preview' as preview_url
                                                       from nft n
                                                                join nft_metadata nm on n.address = nm.nft
                                                       where n.collection = c.address
                                                         and not n.burned
                                                       limit 8 ) ag
                                                order by random()
                                                limit 3 ) ag2 ) previews on true
            where (c.owner = any ($3) or array_length($3::varchar[], 1) is null)
              and ($4::boolean is false or c.verified is true)
              and ($5::varchar is null or c.name ilike $5)
              and (c.address = any ($6) or array_length($6::varchar[], 1) is null)
              and ($7::varchar is null or c.address in (select distinct nсt.collection_address from collection_type_mv nсt
                                where nсt.mimetype = $7 and ($4::boolean is false or nсt.verified is true))
              )
            order by {order} nulls last
            limit $1 offset $2
            "#
        );

        sqlx::query_as(&query)
            .bind(params.limit as i64)
            .bind(params.offset as i64)
            .bind(params.owners)
            .bind(params.verified)
            .bind(params.name)
            .bind(params.collections)
            .bind(params.nft_type)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_roots(&self) -> sqlx::Result<Vec<RootRecord>> {
        sqlx::query_as!(
            RootRecord,
            r#"
            select r.address as "address!", r.code::text as "code!"
            from roots r
            where expiry_date is null
               or now()::timestamp < expiry_date;
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
            select c.address                                                                   as "address!",
                   c.name,
                   c.description,
                   c.logo,
                   c.verified                                                                  as "verified!",
                   case when $3::boolean is false then c.total_count else c.verified_count end as "cnt!",
                   c.nft_count                                                                 as "nft_count!"
            from nft_collection_details c
            where ($3::boolean is false or c.verified is true)
              and ($4::varchar is null or c.name ilike $4)
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

    pub async fn collection_evaluation(
        &self,
        collections: &[String],
        start_timestamp: Option<NaiveDateTime>,
        cutoff_timestamp: Option<NaiveDateTime>,
    ) -> sqlx::Result<Vec<NftCollectionEvaluation>> {
        sqlx::query_as!(
            NftCollectionEvaluation,
            r#"
            with trades as (
                select collection, nft, max_bid as token_amount, price_token as token_root, finished_at as timestamp
                from nft_auction
                where status = 'completed'

                union all

                select collection, nft, price as token_amount, price_token as token_root, finished_at as timestamp
                from nft_direct_buy
                where state = 'filled'

                union all

                select collection, nft, price as token_amount, price_token as token_root, finished_at as timestamp
                from nft_direct_sell
                where state = 'filled'
            ), ranked_trades as (
                select *, row_number() over (partition by nft order by timestamp desc) as row_num
                from trades
                where (timestamp > $2 or $2 is null)
                  and timestamp < coalesce($3, now()::timestamp with time zone)
            ), nft_valuation as (
                select rt.collection,
                       nft,
                       timestamp,
                       token_amount,
                       token_root,
                       tup.usd_price,
                       row_num = 1 as latest
                from nft n
                         join ranked_trades rt on rt.nft = n.address
                         left join token_usd_prices tup on tup.token = rt.token_root
                where not n.burned
            )
            select c.address,
                   coalesce(sum(coalesce(token_amount * usd_price, 0)) filter (where latest), 0) as "usd_value!",
                   coalesce(max(coalesce(token_amount * usd_price, 0)) filter (where latest), 0) as "most_expensive_item!",
                   sum(coalesce(token_amount * usd_price, 0)) as "usd_turnover!",
                   coalesce(ncd.nft_count, 0) as "nft_count!"
            from nft_collection c
                     left join nft_valuation nv on nv.collection = c.address
                     left join nft_collection_details ncd on ncd.address = c.address
            where c.address = any($1)
            group by c.address, ncd.nft_count
            order by 2 desc
            "#,
            collections,
            start_timestamp as _,
            cutoff_timestamp as _,
        )
            .fetch_all(self.db.as_ref())
            .await
    }
}
