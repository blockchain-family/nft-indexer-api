use super::*;
use crate::handlers::{AttributeFilter, CollectionListOrder, NFTListOrder, OrderDirection};
use crate::{handlers::AuctionsSortOrder, token::TokenDict};
use chrono::NaiveDateTime;
use sqlx::{self, postgres::PgPool};
use std::fmt::Write;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Queries {
    db: Arc<PgPool>,
    pub tokens: TokenDict,
}

impl Queries {
    pub fn new(db: Arc<PgPool>, tokens: TokenDict) -> Self {
        Self { db, tokens }
    }

    pub async fn search_all(&self, search_str: &String) -> sqlx::Result<Vec<SearchResult>> {
        sqlx::query_as!(
            SearchResult,
            r#"
                select ag.address as "address!", nft_name, collection_name, object_type as "object_type!", image
                from (
                     select n.address,
                            n.name     nft_name,
                            nc.name    collection_name,
                            'nft'   as object_type,
                            CASE
                                WHEN m.meta is not null THEN m.meta::jsonb -> 'preview' ->> 'source'
                                END as "image",
                            case
                                when lower(n.address) = lower($1) then 10
                                when lower(n.name) = lower($1) then 9
                                when n.name like '' || $1 || ' %' then 7.9
                                when n.name like '% ' || $1 || '' then 7.86
                                when n.name like '%' || $1 || '' then 7.855
                                when n.name like '' || $1 || '%' then 7.85
                                when n.name like '% ' || $1 || ' %' then 7.7
                                when n.name like '%' || $1 || '%' then 7
                                when n.address ilike '%' || $1 || '%' then 5
                                else 1
                                end    priority
                     from nft n
                              left join nft_metadata m on n.address = m.nft
                              join nft_collection nc on n.collection = nc.address and nc.verified
                     where (n.name ilike '%' || $1 || '%'
                         or n.description ilike '%' || $1 || '%'
                         or n.address ilike '%' || $1 || '%')
                       and not n.burned

                     union all

                     select c.address,
                            null            nft_name,
                            c.name          collection_name,
                            'collection' as object_type,
                            c.logo          "image",
                            case
                                when lower(c.address) = lower($1) then 20
                                when lower(c.name) = lower($1) then 19
                                when c.name like '' || $1 || ' %' then 8.9
                                when c.name like '% ' || $1 || '' then 8.86
                                when c.name like '%' || $1 || '' then 8.855
                                when c.name like '' || $1 || '%' then 8.85

                                when c.name like '% ' || $1 || ' %' then 8.7
                                when c.address ilike '%' || $1 || '%' then 6
                                else 2
                                end         priority
                     from nft_collection c
                     where (c.name ilike '%' || $1 || '%'
                         or c.description ilike '%' || $1 || '%'
                         or c.address ilike '%' || $1 || '%')
                       and c.verified
                     ) ag
                order by ag.priority desc
                limit 20
            "#,
            search_str
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn get_nft_details(&self, address: &String) -> sqlx::Result<Option<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
                SELECT n.*, 1::bigint as "total_count!"
                FROM nft_details n
                WHERE n.address = $1
            "#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn get_nft_meta(&self, address: &String) -> sqlx::Result<Option<NftMeta>> {
        sqlx::query_as!(
            NftMeta,
            "SELECT * FROM nft_metadata WHERE nft_metadata.nft = $1",
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

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

    pub async fn get_direct_sell(&self, address: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(
            NftDirectSell,
            r#"
            select s.address             as "address!",
                   s.created             as "created!",
                   s.updated             as "updated!",
                   s.tx_lt               as "tx_lt!",
                   s.nft                 as "nft!",
                   s.collection          as "collection?",
                   s.seller              as "seller?",
                   s.price_token         as "price_token!",
                   s.price               as "price!",
                   s.price * p.usd_price as "usd_price?",
                   s.finished_at         as "finished_at?",
                   s.expired_at          as "expired_at?",
                   case when s.state = 'active'::direct_sell_state and to_timestamp(0) < s.expired_at and s.expired_at < now()::timestamp
                            then 'expired'::direct_sell_state
                        else s.state end as "state!: _",
                   count(1) over ()      as "cnt!",
                   fee_numerator,
                   fee_denominator
            from nft_direct_sell s
                     join offers_whitelist ow on ow.address = s.address
                     left join token_usd_prices p on s.price_token = p.token
                     left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                         from nft_events ne
                                         where ne.event_type = 'market_fee_changed'
                                           and ne.args ->> 'auction' = s.address ) as ev on true
            where s.address = $1
            "#,
            address
        )
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_nft_direct_sell(&self, nft: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(
            NftDirectSell,
            r#"
            select s.address             as "address!",
                   s.created             as "created!",
                   s.updated             as "updated!",
                   s.tx_lt               as "tx_lt!",
                   s.nft                 as "nft!",
                   s.collection          as "collection?",
                   s.seller              as "seller?",
                   s.price_token         as "price_token!",
                   s.price               as "price!",
                   s.price * p.usd_price as "usd_price?",
                   s.finished_at         as "finished_at?",
                   s.expired_at          as "expired_at?",
                   case when s.state = 'active'::direct_sell_state and to_timestamp(0) < s.expired_at and s.expired_at < now()::timestamp
                            then 'expired'::direct_sell_state
                        else s.state end as "state!: _",
                   count(1) over ()      as "cnt!",
                   fee_numerator,
                   fee_denominator
            from nft_direct_sell s
                     join offers_whitelist ow on ow.address = s.address
                     left join token_usd_prices p on s.price_token = p.token
                     left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                         from nft_events ne
                                         where ne.event_type = 'market_fee_changed'
                                           and ne.args ->> 'auction' = s.address ) as ev on true
            where s.nft = $1
              and s.state in ('active', 'expired')
            order by s.created desc
            limit 1
            "#,
            nft
        )
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_direct_buy(&self, address: &String) -> sqlx::Result<Option<NftDirectBuy>> {
        sqlx::query_as!(
            NftDirectBuy,
            r#"
        SELECT
        s.address as "address!",
        s.created as "created!",
        s.updated as "updated!",
        s.tx_lt as "tx_lt!",
        s.nft as "nft!",
        s.collection,
        s.buyer,
        s.price_token as "price_token!",
        s.price as "price!",
        s.usd_price,
        s.finished_at,
        s.expired_at,
        s.state as "state!: _",
        1::bigint as "cnt!",
        s.fee_numerator,
        s.fee_denominator
        FROM nft_direct_buy_usd s
        WHERE s.address = $1"#,
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
                   c.total_count as "cnt!"
            from nft_collection_details c
            where c.address = any ($1)
              --and owner is not null
             "#,
            ids
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_nfts(&self, ids: &[String]) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
                SELECT n.*, 1::bigint as "total_count!"
                FROM nft_details n
                WHERE n.address = ANY($1)
            "#,
            ids
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn collect_auctions(&self, ids: &[String]) -> sqlx::Result<Vec<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
            with a as (
                select distinct on (a.address) a.address,
                                   a.nft,
                                   a.collection,
                                   a.nft_owner,
                                   a.wallet_for_bids,
                                   a.price_token,
                                   a.start_price,
                                   a.max_bid,
                                   a.min_bid,
                                   case
                                       when a.status = 'active'::auction_status and to_timestamp(0) < a.finished_at and
                                            a.finished_at < now()::timestamp then 'expired'::auction_status
                                       else a.status end                          as "status: _",
                                   a.created_at,
                                   a.finished_at,
                                   a.tx_lt,
                                   sum(case when b.auction is null then 0 else 1 end)
                                   over (partition by a.address)                  as bids_count,
                                   first_value(b.buyer) over bids_w               as last_bid_from,
                                   first_value(b.price) over bids_w               as last_bid_value,
                                   first_value(b.price * p.usd_price) over bids_w as last_bid_usd_value,
                                   first_value(b.created_at) over bids_w          as last_bid_ts,
                                   a.start_price * p.usd_price                    as start_usd_price,
                                   a.max_bid * p.usd_price                        as max_usd_bid,
                                   a.min_bid * p.usd_price                        as min_usd_bid,
                                   ev.fee_numerator,
                                   ev.fee_denominator
                                from nft_auction a
                                         join offers_whitelist ow on ow.address = a.address
                                         left join nft_auction_bid b on b.auction = a.address and b.declined is false
                                         left join token_usd_prices p on p.token = a.price_token
                                         left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                                    (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                                             from nft_events ne
                                                             where ne.event_type = 'market_fee_changed'
                                                               and ne.args ->> 'auction' = a.address ) as ev on true
                                where a.address = any ($1)
                                  and (
                                        b.declined is false
                                        or b.declined is null
                                    )
                                    window bids_w as (partition by b.auction order by b.created_at desc)
                            )
                            select a.address               as "address?",
                                   a.nft                   as "nft?",
                                   a.wallet_for_bids       as "wallet_for_bids?",
                                   a.price_token           as "price_token?",
                                   a.start_price,
                                   a.max_bid,
                                   a.min_bid,
                                   a.start_usd_price,
                                   a.max_usd_bid,
                                   a.min_usd_bid,
                                   "status: _",
                                   a.created_at,
                                   a.finished_at,
                                   a.tx_lt                 as "tx_lt?",
                                   a.bids_count,
                                   a.last_bid_from,
                                   a.last_bid_ts,
                                   a.last_bid_value,
                                   a.last_bid_usd_value,
                                   a.fee_numerator,
                                   a.fee_denominator,
                                   count(1) over ()        as "cnt!"
                            from a
            "#,
            ids
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_direct_buy(&self, ids: &[String]) -> sqlx::Result<Vec<NftDirectBuy>> {
        sqlx::query_as!(
            NftDirectBuy,
            r#"SELECT
            s.address as "address!",
            s.created as "created!",
            s.updated as "updated!",
            s.tx_lt as "tx_lt!",
            s.nft as "nft!",
            s.collection,
            s.buyer,
            s.price_token as "price_token!",
            s.price as "price!",
            s.usd_price,
            s.finished_at,
            s.expired_at,
            s.state as "state!: _",
            1::bigint as "cnt!",
            s.fee_numerator,
            s.fee_denominator
            FROM nft_direct_buy_usd s
            WHERE s.address = ANY($1)"#,
            ids
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn collect_direct_sell(&self, ids: &[String]) -> sqlx::Result<Vec<NftDirectSell>> {
        sqlx::query_as!(
            NftDirectSell,
            r#"
            select s.address             as "address!",
                   s.created             as "created!",
                   s.updated             as "updated!",
                   s.tx_lt               as "tx_lt!",
                   s.nft                 as "nft!",
                   s.collection          as "collection?",
                   s.seller              as "seller?",
                   s.price_token         as "price_token!",
                   s.price               as "price!",
                   s.price * p.usd_price as "usd_price?",
                   s.finished_at         as "finished_at?",
                   s.expired_at          as "expired_at?",
                   case when s.state = 'active'::direct_sell_state and to_timestamp(0) < s.expired_at and s.expired_at < now()::timestamp
                            then 'expired'::direct_sell_state
                        else s.state end as "state!: _",
                   count(1) over ()      as "cnt!",
                   fee_numerator,
                   fee_denominator
            from nft_direct_sell s
                     join offers_whitelist ow on ow.address = s.address
                     left join token_usd_prices p on s.price_token = p.token
                     left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                         from nft_events ne
                                         where ne.event_type = 'market_fee_changed'
                                           and ne.args ->> 'auction' = s.address ) as ev on true
            where s.address = any ($1)
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
                   c.total_count as "cnt!"
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
            order by {order} nulls last
            limit $1 offset $2
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
            select r.address as "address!", r.code::text as "code!"
            from roots r
            where expiry_date is null
               or now()::timestamp < expiry_date
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

    pub async fn nft_top_search(
        &self,
        from: NaiveDateTime,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
            with nfts as (
                    select nvm.address, count(1) as subcnt, nvm.updated
                    from nft_verified_extended nvm
                             left join nft_price_history nph
                                       on nvm.address = nph.nft
                                           and nph.ts >= $1
                             join offers_whitelist ow on ow.address = nph.source

                    where nvm.updated >= $1
                    group by nvm.address, nvm.updated, nvm.address
                    having count(1) > 0
                    order by count(1) desc, nvm.updated desc, nvm.address desc
                    limit $2 offset $3
                )
                select n.address as "address?",
                       n.collection as "collection?",
                       n.owner as "owner?",
                       n.manager as "manager?",
                       n.name::text                         as "name?",
                       n.description as "description?",
                       n.burned as "burned?",
                       n.updated as "updated?",
                       n.owner_update_lt                    as "tx_lt?",
                       m.meta as "meta?",
                       auc.auction as "auction?",
                       auc."auction_status: _",
                       sale.forsale as "forsale?",
                       sale."forsale_status: _",
                       (select distinct on (s.address) first_value(s.address) over w
                        from nft_direct_buy s
                                 left join token_usd_prices tup on tup.token = s.price_token
                        where state = 'active'
                          and nft = n.address
                            window w as (partition by nft order by s.price * tup.usd_price desc)
                        limit 1)                            as "best_offer?",
                       least(auc.price_usd, sale.price_usd) as "floor_price_usd?",
                       last_deal.last_price                 as "deal_price_usd?",
                       case
                           when least(auc.price_usd, sale.price_usd) = auc.price_usd then auc.min_bid
                           when least(auc.price_usd, sale.price_usd) = sale.price_usd then sale.price
                           else null::numeric end           as "floor_price?",
                       case
                           when least(auc.price_usd, sale.price_usd) = auc.price_usd
                               then auc.token::character varying
                           when least(auc.price_usd, sale.price_usd) = sale.price_usd
                               then sale.token::character varying
                           else null::character varying end as "floor_price_token?",
                       n.id::text                           as "nft_id?",
                       count(1) over ()                          as "total_count!"
                from nft n
                         join nfts
                              on nfts.address = n.address
                         left join lateral ( select nph.price * tup.usd_price as last_price
                                             from nft_price_history nph
                                                      join offers_whitelist ow on ow.address = nph.source
                                                      left join token_usd_prices tup on tup.token = nph.price_token
                                             where nph.nft = n.address
                                             order by nph.ts desc
                                             limit 1 ) last_deal on true
                         left join lateral ( select a.address                 as auction,
                                                    case
                                                        when a.status = 'active' and
                                                             to_timestamp(0) < a.finished_at and
                                                             a.finished_at < now() then 'expired'
                                                        else a.status end     as "auction_status: _",
                                                    a.min_bid * tup.usd_price as price_usd,
                                                    tup.token,
                                                    a.min_bid
                                             from nft_auction a
                                                      join offers_whitelist ow on ow.address = a.address
                                                      left join token_usd_prices tup on tup.token = a.price_token
                                             where a.nft = n.address
                                               and a.status in ('active', 'expired')
                                             limit 1 ) auc on true
                         left join nft_metadata m on m.nft = n.address
                         left join lateral ( select s.address               as forsale,
                                                    case
                                                        when s.state = 'active' and
                                                             to_timestamp(0) < s.expired_at and s.expired_at < now()
                                                            then 'expired'
                                                        else s.state end    as "forsale_status: _",
                                                    s.price * tup.usd_price as price_usd,
                                                    s.price,
                                                    tup.token
                                             from nft_direct_sell s
                                                      join offers_whitelist ow on ow.address = s.address
                                                      left join token_usd_prices tup on tup.token = s.price_token
                                             where s.nft = n.address
                                               and s.state in ('active', 'expired')
                                             limit 1 ) sale on true
                where not n.burned
                order by nfts.subcnt desc, nfts.updated desc, nfts.address desc
            "#,
            from,
            limit,
            offset
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn nft_search(
        &self,
        owners: &[Address],
        collections: &[Address],
        _price_from: Option<u64>,
        _price_to: Option<u64>,
        _price_token: Option<Address>,
        forsale: Option<bool>,
        auction: Option<bool>,
        verified: Option<bool>,
        limit: usize,
        offset: usize,
        attributes: &Vec<AttributeFilter>,
        order: Option<NFTListOrder>,
        with_count: bool,
    ) -> sqlx::Result<Vec<NftDetails>> {
        let mut sql = r#"
            SELECT n.*,
            n."auction_status: _" as auction_status,
            n."forsale_status: _" as forsale_status,
            case when $8 then
                count(1) over ()
            else 0
                end total_count
            FROM nft_details n
            INNER JOIN nft_collection c ON n.collection = c.address
            WHERE
            (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
            and (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            and n.burned = false
            and (($3::bool is null and $4::bool is null)
                or ($3::bool is not null and $4::bool is not null
                    and (($4::bool and n.forsale is not null and n."forsale_status: _" = 'active') or (not $4::bool and n.forsale is null)
                    or ($3::bool and n.auction is not null and n."auction_status: _" = 'active') or (not $3::bool and n.auction is null))
                )
                or (
                    $3::bool is null
                    and (($4::bool and n.forsale is not null and n."forsale_status: _" = 'active') or (not $4::bool and n.forsale is null))
                )
                or (
                    $4::bool is null
                    and (($3::bool and n.auction is not null and n."auction_status: _" = 'active') or (not $3::bool and n.auction is null))
                )
            )
            and ($5::boolean is false OR c.verified is true)
        "#.to_string();

        for attribute in attributes {
            let values = attribute
                .trait_values
                .iter()
                .enumerate()
                .map(|it| format!("'{}'", it.1.to_lowercase()))
                .collect::<Vec<String>>()
                .join(",");
            // TODO need index trim(na.value #>> '{}')
            let _ = write!(
                sql,
                r#" and exists(
                select 1 from nft_attributes na
                where
                    na.nft = n.address and (lower(na.trait_type) = lower('{0}') and lower(trim(na.value #>> '{{}}')) in ({1}))
                )
            "#,
                attribute.trait_type, values
            );
        }

        match order {
            None => {
                let _ = write!(
                    sql,
                    r#"
                        ORDER BY n.name, n.address
                    "#
                );
            }
            Some(order) => {
                let field = order.field.to_string();

                match order.direction {
                    OrderDirection::Asc => {
                        let _ = write!(sql, "order by n.{field}");
                    }
                    OrderDirection::Desc => {
                        let _ = write!(sql, "order by coalesce(n.{field}, 0) desc");
                    }
                }
            }
        };

        let _ = write!(
            sql,
            r#"
                LIMIT $6 OFFSET $7
            "#
        );

        sqlx::query_as(&sql)
            .bind(owners)
            .bind(collections)
            .bind(auction)
            .bind(forsale)
            .bind(verified)
            .bind(limit as i64)
            .bind(offset as i64)
            .bind(with_count)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn get_nft_auction(&self, address: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
             with auction as ( select distinct on (a.address) a.address,
                                                             a.nft,
                                                             a.collection,
                                                             a.nft_owner,
                                                             a.wallet_for_bids,
                                                             a.price_token,
                                                             a.start_price,
                                                             a.max_bid,
                                                             a.min_bid,
                                                             case when a.status = 'active'::auction_status and
                                                                       to_timestamp(0) < a.finished_at and a.finished_at < now()::timestamp
                                                                      then 'expired'::auction_status
                                                                  else a.status end                         as "status: _",
                                                             a.created_at,
                                                             a.finished_at,
                                                             a.tx_lt,
                                                             sum(case when b.auction is null then 0 else 1 end)
                                                             over (partition by a.address)                  as bids_count,
                                                             first_value(b.buyer) over bids_w               as last_bid_from,
                                                             first_value(b.price) over bids_w               as last_bid_value,
                                                             first_value(b.price * p.usd_price) over bids_w as last_bid_usd_value,
                                                             first_value(b.created_at) over bids_w          as last_bid_ts,
                                                             a.start_price * p.usd_price                    as start_usd_price,
                                                             a.max_bid * p.usd_price                        as max_usd_bid,
                                                             a.min_bid * p.usd_price                        as min_usd_bid,
                                                             ev.fee_numerator,
                                                             ev.fee_denominator
                              from nft_auction a
                                       join offers_whitelist ow on ow.address = a.address
                                       left join nft_auction_bid b on b.auction = a.address and b.declined is false
                                       left join token_usd_prices p on p.token = a.price_token
                                       left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                                  (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                                           from nft_events ne
                                                           where ne.event_type = 'market_fee_changed'
                                                             and ne.args ->> 'auction' = a.address ) as ev on true
                              where (b.declined is false or b.declined is null)
                                and a.address = $1
                              window bids_w as (partition by b.auction order by b.created_at desc) )

            select a.address            as "address?",
                   a.nft                as "nft?",
                   a.wallet_for_bids    as "wallet_for_bids?",
                   a.price_token        as "price_token?",
                   a.start_price        as "start_price?",
                   a.max_bid            as "max_bid?",
                   a.min_bid            as "min_bid?",
                   a.start_usd_price    as "start_usd_price?",
                   a.max_usd_bid        as "max_usd_bid?",
                   a.min_usd_bid        as "min_usd_bid?",
                   "status: _",
                   a.created_at         as "created_at?",
                   a.finished_at        as "finished_at?",
                   a.tx_lt              as "tx_lt?",
                   a.bids_count         as "bids_count?",
                   a.last_bid_from      as "last_bid_from?",
                   a.last_bid_ts        as "last_bid_ts?",
                   a.last_bid_value     as "last_bid_value?",
                   a.last_bid_usd_value as "last_bid_usd_value?",
                   a.fee_numerator      as "fee_numerator?",
                   a.fee_denominator    as "fee_denominator?",
                   count(1) over ()     as "cnt!"
            from auction a
            "#,
            address
        )
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_traits(&self, nft: &Address) -> sqlx::Result<Vec<NftTraitRecord>> {
        sqlx::query_as!(
            NftTraitRecord,
            r#"
                WITH nft_attributes AS (
                    SELECT jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' AS trait_type,
                           jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      AS trait_value,
                           nm.meta,
                           n.collection                                                  AS nft_collection,
                           nm.nft
                    FROM nft_metadata nm
                             JOIN nft n ON n.address = nm.nft
                    WHERE nm.meta -> 'attributes' IS NOT NULL
                      AND nm.nft = $1
                ),
                     nft_attributes_col AS (
                         SELECT jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' AS trait_type,
                                jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      AS trait_value,
                                nm.nft
                         FROM nft_metadata nm
                         where nm.nft in (
                             select n2.address
                             from nft n2
                                      join nft n3 on n3.address = $1
                                 and n2.collection = n3.collection
                         )
                     )
                SELECT (na.trait_type #>> '{}')::text as trait_type, (na.trait_value #>> '{}')::text as trait_value, COUNT(*) as "cnt!"
                FROM nft_attributes na
                         LEFT JOIN nft_attributes_col na2
                                   ON na.trait_type = na2.trait_type
                                       AND na.trait_value = na2.trait_value
                GROUP BY na.trait_type, na.trait_value
            "#,
            nft
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn get_nft_auction_last_bid(
        &self,
        auction: &String,
    ) -> sqlx::Result<Option<NftAuctionBid>> {
        sqlx::query_as!(
            NftAuctionBid,
            r#"
            select first_value(b.auction) over w                        as "auction!",
                   first_value(b.buyer) over w                          as "buyer!",
                   first_value(b.price) over w                          as "price!",
                   first_value(b.price * tup.usd_price) over w          as "usd_price",
                   first_value(b.created_at) over w                     as "created_at!",
                   first_value(b.next_bid_value) over w                 as "next_bid_value!",
                   first_value(b.next_bid_value * tup.usd_price) over w as "next_bid_usd_value",
                   first_value(b.tx_lt) over w                          as "tx_lt!",
                   true                                                 as "active!",
                   count(1) over ()                                     as "cnt!"
            from nft_auction_bid b
                     join offers_whitelist ow on ow.address = b.auction
                     left join token_usd_prices tup on tup.token = b.price_token
            where auction = $1
              and declined is false
            window w as (partition by auction order by created_at desc)
            limit 1
            "#,
            auction
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn list_nft_auction_bids(
        &self,
        auction: &String,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuctionBid>> {
        sqlx::query_as!(
            NftAuctionBid,
            r#"
            select b.auction                             as "auction!",
                   b.buyer                               as "buyer!",
                   b.price                               as "price!",
                   b.price * tup.usd_price               as "usd_price",
                   b.created_at                          as "created_at!",
                   b.next_bid_value                      as "next_bid_value!",
                   b.next_bid_value * tup.usd_price      as "next_bid_usd_value",
                   b.tx_lt                               as "tx_lt!",
                   max(created_at) over w = b.created_at as "active!",
                   count(1) over ()                      as "cnt!"
            from nft_auction_bid b
                     join offers_whitelist ow on ow.address = b.auction
                     left join token_usd_prices tup on tup.token = b.price_token
            where auction = $1
              and declined is false
            window w as (partition by auction)
            order by created_at desc
            limit $2 offset $3
            "#,
            auction,
            limit as i64,
            offset as i64
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_nft_auctions(
        &self,
        owners: &[Address],
        collections: &[Address],
        tokens: &[Address],
        sort: &AuctionsSortOrder,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuction>> {
        let query = r#"
        with a as (
                select distinct on (a.address) a.address,
                                               a.nft,
                                               a.collection,
                                               a.nft_owner,
                                               a.wallet_for_bids,
                                               a.price_token,
                                               a.start_price,
                                               a.max_bid,
                                               a.min_bid,
                                               case
                                                   when a.status = 'active'::auction_status and to_timestamp(0) < a.finished_at and
                                                        a.finished_at < now()::timestamp then 'expired'::auction_status
                                                   else a.status end                          as status,
                                               a.created_at,
                                               a.finished_at,
                                               a.tx_lt,
                                               sum(case when b.auction is null then 0 else 1 end)
                                               over (partition by a.address)                  as bids_count,
                                               first_value(b.buyer) over bids_w               as last_bid_from,
                                               first_value(b.price) over bids_w               as last_bid_value,
                                               first_value(b.price * p.usd_price) over bids_w as last_bid_usd_value,
                                               first_value(b.created_at) over bids_w          as last_bid_ts,
                                               a.start_price * p.usd_price                    as start_usd_price,
                                               a.max_bid * p.usd_price                        as max_usd_bid,
                                               a.min_bid * p.usd_price                        as min_usd_bid,
                                               ev.fee_numerator,
                                               ev.fee_denominator,
                                               count(1) over () as cnt
                from nft_auction a
                         join offers_whitelist ow on ow.address = a.address
                         left join nft_auction_bid b on b.auction = a.address and b.declined is false
                         left join token_usd_prices p on p.token = a.price_token
                         left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                    (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                             from nft_events ne
                                             where ne.event_type = 'market_fee_changed'
                                               and ne.args ->> 'auction' = a.address ) as ev on true
                where ((a.nft_owner = any ($1) or array_length($1::varchar[], 1) is null)
                    and (a.collection = any ($2) or array_length($2::varchar[], 1) is null)
                    and (a.nft = any ($3) or array_length($3::varchar[], 1) is null))
                  and (
                        b.declined is false
                        or b.declined is null)
                    window bids_w as (partition by b.auction order by b.created_at desc)
            )

            select a.address,
                   a.nft,
                   a.wallet_for_bids,
                   a.price_token,
                   a.start_price,
                   a.max_bid,
                   a.min_bid,
                   a.start_usd_price,
                   a.max_usd_bid,
                   a.min_usd_bid,
                   a.status,
                   a.created_at,
                   a.finished_at,
                   a.tx_lt,
                   a.bids_count,
                   a.last_bid_from,
                   a.last_bid_ts,
                   a.last_bid_value,
                   a.last_bid_usd_value,
                   a.fee_numerator,
                   a.fee_denominator,
                   a.cnt
            from a
        "#;

        let (order_by, order_direction) = match sort {
            AuctionsSortOrder::BidsCount => ("bids_count", ""),
            _ => ("created_at", "desc"),
        };

        let full_query = format!(
            "{} ORDER BY {} {} LIMIT $4 OFFSET $5",
            query, order_by, order_direction
        );

        sqlx::query_as(full_query.as_str())
            .bind(owners)
            .bind(collections)
            .bind(tokens)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_events(
        &self,
        nft: Option<&String>,
        collections: &[String],
        owner: Option<&String>,
        event_type: &[NftEventType],
        offset: usize,
        limit: usize,
        with_count: bool,
        verified: Option<bool>,
    ) -> sqlx::Result<NftEventsRecord> {
        let event_types_slice = &event_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];

        sqlx::query_file_as!(
            NftEventsRecord,
            "src/db/sql/list_activities.sql",
            event_types_slice as _,
            owner as _,
            nft as _,
            collections as _,
            limit as i64,
            offset as i64,
            with_count,
            verified
        )
        .fetch_one(self.db.as_ref())
        .await
    }

    pub async fn list_events_count(
        &self,
        nft: Option<&String>,
        collection: Option<&String>,
        _owner: Option<&String>,
        typ: &[EventType],
    ) -> sqlx::Result<i64> {
        let typ_str: Vec<String> = typ.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "
            SELECT count(*)
            FROM nft_events e
            WHERE
                ($1::varchar is null OR e.nft = $1)
                AND ($2::varchar is null OR e.collection = $2)
                AND (array_length($3::varchar[], 1) is null OR e.event_type::varchar = ANY($3))
            ",
            nft,
            collection,
            &typ_str
        )
        .fetch_one(self.db.as_ref())
        .await
        .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_nft_direct_buy(
        &self,
        nft: &String,
        status: &[DirectBuyState],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDirectBuy>> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(
            NftDirectBuy,
            r#"
            select s.address             as "address!",
                   s.created             as "created!",
                   s.updated             as "updated!",
                   s.tx_lt               as "tx_lt!",
                   s.nft                 as "nft!",
                   s.collection          as "collection?",
                   s.buyer               as "buyer?",
                   s.price_token         as "price_token!",
                   s.price               as "price!",
                   s.price * p.usd_price as "usd_price?",
                   s.finished_at         as "finished_at?",
                   s.expired_at          as "expired_at?",
                   case when s.state = 'active'::direct_buy_state and to_timestamp(0::double precision) < s.expired_at and
                             s.expired_at < now()::timestamp then 'expired'::direct_buy_state
                        else s.state end as "state!: _",
                   count(1) over ()      as "cnt!",
                   fee_numerator,
                   fee_denominator
            from nft_direct_buy s
                     join offers_whitelist ow on ow.address = s.address
                     left join token_usd_prices p on s.price_token = p.token
                     left join lateral ( select ((ne.args -> 'fee') -> 'numerator')::integer   as fee_numerator,
                                                ((ne.args -> 'fee') -> 'denominator')::integer as fee_denominator
                                         from nft_events ne
                                         where ne.event_type = 'market_fee_changed'::event_type
                                           and (ne.args ->> 'auction') = s.address) ev on true
            where s.nft = $1
              and s.state = 'active'::direct_buy_state
              and (to_timestamp(0::double precision) = s.expired_at or s.expired_at > now()::timestamp)
              and (array_length($2::varchar[], 1) is null or s.state::varchar = any ($2))
            order by s.updated desc
            limit $3 offset $4
            "#,
            nft,
            &status_str,
            limit as i64,
            offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_sell(
        &self,
        owner: &String,
        collections: &[String],
        status: &[DirectSellState],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDirectSell>> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(
            NftDirectSell,
            r#"
        select s.address             as "address!",
                   s.created             as "created!",
                   s.updated             as "updated!",
                   s.tx_lt               as "tx_lt!",
                   s.nft                 as "nft!",
                   s.collection          as "collection?",
                   s.seller              as "seller?",
                   s.price_token         as "price_token!",
                   s.price               as "price!",
                   s.price * p.usd_price as "usd_price?",
                   s.finished_at         as "finished_at?",
                   s.expired_at          as "expired_at?",
                   s.state               as "state!: _",
                   count(1) over ()      as "cnt!",
                   fee_numerator,
                   fee_denominator
            from nft_direct_sell s
                     join offers_whitelist ow on ow.address = s.address
                     left join token_usd_prices p on s.price_token = p.token
                     left join lateral ( select (ne.args -> 'fee' -> 'numerator')::int   as fee_numerator,
                                                (ne.args -> 'fee' -> 'denominator')::int as fee_denominator
                                         from nft_events ne
                                         where ne.event_type = 'market_fee_changed'
                                           and ne.args ->> 'auction' = s.address ) as ev on true
            where s.seller = $1
              and (s.collection = any ($2) or array_length($2::varchar[], 1) is null)
              and (array_length($3::varchar[], 1) is null or s.state::varchar = any ($3))
            order by s.updated desc
            limit $4 offset $5
            "#,
            owner,
            collections,
            &status_str,
            limit as i64,
            offset as i64
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_buy(
        &self,
        owner: &String,
        collections: &[String],
        status: &[DirectBuyState],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDirectBuy>> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(NftDirectBuy, r#"
        SELECT s.address as "address!", s.created as "created!", s.updated as "updated!", s.tx_lt as "tx_lt!",
        s.nft as "nft!", s.collection,
        s.buyer,
        s.price_token as "price_token!", s.price as "price!",
        s.usd_price,
        s.finished_at, s.expired_at,
        s.state as "state!: _",
        count(1) over () as "cnt!",
        s.fee_numerator,
        s.fee_denominator
        FROM nft_direct_buy_usd s
        WHERE s.buyer = $1
            AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_buy_in(
        &self,
        owner: &String,
        collections: &[String],
        status: &[DirectBuyState],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDirectBuy>> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(NftDirectBuy, r#"
        SELECT s.address as "address!", s.created as "created!", s.updated as "updated!", s.tx_lt as "tx_lt!",
        s.nft as "nft!", s.collection,
        s.buyer,
        s.price_token as "price_token!", s.price as "price!",
        s.usd_price,
        s.finished_at, s.expired_at,
        s.state as "state!: _",
        count(1) over () as "cnt!",
        s.fee_numerator,
        s.fee_denominator
        FROM nft_direct_buy_usd s
        INNER JOIN nft n ON n.address = s.nft
        WHERE n.owner = $1
            AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_auction_bids_out(
        &self,
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuctionBidExt>> {
        sqlx::query_as!(
            NftAuctionBidExt,
            r#"
            with bids_detailed as ( select x.auction                               as "auction!",
                                           x.buyer                                 as "buyer!",
                                           x.price                                 as "price!",
                                           x.price_token,
                                           x.created_at                            as "created_at!",
                                           x.next_bid_value,
                                           x.tx_lt,
                                           max(x.created_at) over w = x.created_at as active,
                                           x.price * tup.usd_price                 as usd_price,
                                           x.next_bid_value * tup.usd_price        as next_bid_usd_value,
                                           x.nft,
                                           x.collection
                                    from nft_auction_bid x
                                             join offers_whitelist ow on ow.address = x.auction
                                             left join token_usd_prices tup on tup.token = x.price_token
                                    where x.buyer = $1
                                       and (x.collection = any ($2) or array_length($2::varchar[], 1) is null)
                                    window w as (partition by x.auction) )
            select "auction!",
                   "buyer!",
                   "price!",
                   price_token        as "price_token?",
                   "created_at!",
                   next_bid_value     as "next_bid_value?",
                   tx_lt              as "tx_lt?",
                   active             as "active?",
                   usd_price          as "usd_price?",
                   next_bid_usd_value as "next_bid_usd_value?",
                   nft                as "nft?",
                   collection         as "collection?",
                   count(1) over () as "cnt!"
            from bids_detailed b
            where ($3::bool is null or $3::bool = false or b.active is true)
            order by b."created_at!" desc
            limit $4 offset $5
            "#,
            owner,
            collections,
            lastbid.clone(),
            limit as i64,
            offset as i64
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_auction_bids_in(
        &self,
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuctionBidExt>> {
        sqlx::query_as!(
            NftAuctionBidExt,
            r#"
            with bids_detailed as ( select x.auction                               as "auction!",
                                           x.buyer                                 as "buyer!",
                                           x.price                                 as "price!",
                                           x.price_token,
                                           x.created_at                            as "created_at!",
                                           x.next_bid_value,
                                           x.tx_lt,
                                           x.nft_owner                             as owner,
                                           max(x.created_at) over w = x.created_at as active,
                                           x.price * tup.usd_price                 as usd_price,
                                           x.next_bid_value * tup.usd_price        as next_bid_usd_value,
                                           x.nft,
                                           x.collection
                                    from nft_auction_bid x
                                             left join token_usd_prices tup on tup.token = x.price_token
                                    where x.nft_owner = $1
                                    and (x.collection = any ($2) or array_length($2::varchar[], 1) is null)
                                    window w as (partition by x.auction) )
            select "auction!",
                   "buyer!",
                   "price!",
                   price_token        as "price_token?",
                   "created_at!",
                   next_bid_value     as "next_bid_value?",
                   tx_lt              as "tx_lt?",
                   active             as "active?",
                   usd_price          as "usd_price?",
                   next_bid_usd_value as "next_bid_usd_value?",
                   nft                as "nft?",
                   collection         as "collection?",
                   count(1) over ()   as "cnt!"
            from bids_detailed x
            where (x.active = true or ($3::bool is null or $3::bool = false))
            order by x."created_at!" desc
            limit $4 offset $5
            "#,
            owner,
            collections,
            lastbid.clone(),
            limit as i64,
            offset as i64
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_nft_price_history(
        &self,
        nft: &str,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> sqlx::Result<Vec<NftPrice>> {
        sqlx::query_as!(
            NftPrice,
            r#"
            select ts, usd_price as "usd_price!"
            from nft_price_history nph
                     inner join offers_whitelist ow on ow.address = nph.source
            where nft = $1
              and ts between $2 and $3
              and usd_price is not null
            order by ts
            "#,
            nft,
            from,
            to,
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_attributes_dictionary(&self) -> sqlx::Result<Vec<TraitDef>> {
        sqlx::query_as!(
            TraitDef,
            r#"
            select a.collection, a.trait_type, jsonb_agg(a.value) as values
            from ( select distinct a.collection, a.trait_type, a.value
                   from nft_attributes a
                   order by a.collection, a.trait_type, a.value ) as a
            group by a.collection, a.trait_type
            "#
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_attributes_search(
        &self,
        collection: &String,
        trait_type: &String,
        values: &[serde_json::Value],
    ) -> sqlx::Result<Vec<String>> {
        sqlx::query!(
            "
        select distinct a.nft
        from nft_attributes a
        where a.collection = $1
        and a.trait_type = $2
        and a.value = ANY($3::jsonb[])
        order by 1 asc",
            collection,
            trait_type,
            values
        )
        .fetch_all(self.db.as_ref())
        .await
        .map(|x| x.iter().map(|y| y.nft.clone()).collect())
    }

    pub async fn update_token_usd_prices(
        &self,
        mut prices: Vec<TokenUsdPrice>,
    ) -> sqlx::Result<()> {
        for price in prices.drain(..) {
            sqlx::query!(
                "
            INSERT INTO token_usd_prices (token, usd_price, ts)
            VALUES ($1::varchar, $2, $3)
            ON CONFLICT (token) DO UPDATE
            SET
                usd_price = EXCLUDED.usd_price,
                ts = EXCLUDED.ts;
            ",
                price.token,
                price.usd_price,
                price.ts
            )
            .execute(self.db.as_ref())
            .await?;
        }
        Ok(())
    }

    pub async fn get_metrics_summary(
        &self,
        from: NaiveDateTime,
        to: NaiveDateTime,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<MetricsSummaryRecord>> {
        sqlx::query_file_as!(
            MetricsSummaryRecord,
            "src/db/sql/metrics_summary.sql",
            from,
            to,
            limit,
            offset
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn get_owner_fee(
        &self,
        owner: &Address,
        root_code: &RootType,
    ) -> sqlx::Result<OwnerFeeRecord> {
        sqlx::query_as!(
            OwnerFeeRecord,
            r#"
            select case when fee.fee_numerator is not null and fee.fee_denominator is not null then fee.fee_numerator
                        else (ne.args -> 'fee' -> 'numerator')::int end   "fee_numerator!",
                   case when fee.fee_numerator is not null and fee.fee_denominator is not null then fee.fee_denominator
                        else (ne.args -> 'fee' -> 'denominator')::int end "fee_denominator!",
                   fee.collection,
                   fee.nft
            from nft_events ne
                     join roots r on ne.address = r.address and r.code = $2::t_root_types
                     left join lateral ( select nc.fee_numerator,
                                                nc.fee_denominator,
                                                max(n.collection) collection,
                                                max(n.id)::text   nft
                                         from nft n
                                                  join nft_collection nc
                                                       on n.collection = nc.address and nc.fee_numerator is not null and
                                                          nc.fee_denominator is not null
                                         where n.owner = $1
                                         group by nc.fee_numerator, nc.fee_denominator
                                         order by min(nc.fee_numerator / nc.fee_denominator)
                                         limit 1 ) as fee on true
            where ne.event_type = 'market_fee_default_changed'
            order by created_at desc, created_lt desc, id desc
            limit 1
            "#,
            owner as &Address,
            root_code as &RootType
        )
            .fetch_one(self.db.as_ref())
            .await
    }
}
