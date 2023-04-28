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
                SELECT c.*, 1::bigint as "cnt!", '[]'::json as "previews!"
                FROM nft_collection_details c
                WHERE c.address = $1"#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn get_direct_sell(&self, address: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(
            NftDirectSell,
            r#"
        SELECT
        s.address as "address!",
        s.created as "created!",
        s.updated as "updated!",
        s.tx_lt as "tx_lt!",
        s.nft as "nft!",
        s.collection,
        s.seller,
        s.price_token as "price_token!",
        s.price as "price!",
        s.usd_price,
        s.finished_at,
        s.expired_at,
        s.state as "state!: _",
         count (1) over () as "cnt!",
        s.fee_numerator,
        s.fee_denominator
        FROM nft_direct_sell_usd s
        WHERE s.address = $1"#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn get_nft_direct_sell(&self, nft: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(
            NftDirectSell,
            r#"
            SELECT
                s.address as "address!",
                s.created as "created!",
                s.updated as "updated!",
                s.tx_lt as "tx_lt!",
                s.nft as "nft!",
                s.collection,
                s.seller,
                s.price_token as "price_token!",
                s.price as "price!",
                s.usd_price,
                s.finished_at,
                s.expired_at,
                s.state as "state!: _",
                count (1) over () as "cnt!",
                s.fee_numerator,
                s.fee_denominator
            FROM nft_direct_sell_usd s
            WHERE s.nft = $1 and s.state in ('active', 'expired')
            ORDER BY s.created DESC LIMIT 1"#,
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
            r#"SELECT a.*, count(1) over() as "cnt!" FROM nft_auction_search a WHERE a.address = ANY($1)"#,
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
            r#"SELECT
            s.address as "address!",
            s.created as "created!",
            s.updated as "updated!",
            s.tx_lt as "tx_lt!",
            s.nft as "nft!",
            s.collection,
            s.seller,
            s.price_token as "price_token!",
            s.price as "price!",
            s.usd_price,
            s.finished_at,
            s.expired_at,
            s.state as "state!: _",
             count (1) over () as "cnt!",
            s.fee_numerator,
            s.fee_denominator
            FROM nft_direct_sell_usd s
            WHERE s.address = ANY($1)"#,
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

    pub async fn nft_top_search(
        &self,
        from: NaiveDateTime,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
            select n.*, count(1) over () as "total_count!"
                from nft_details n
                     join nft_collection nc
                          on nc.address = n.collection
                              and nc.verified
                     left join lateral ( select count(1) as cnt
                                         from nft_auction na
                                          join events_whitelist ew on na.address = ew.address
                                         where n.address = na.nft
                                           and na.status = 'completed'
                                           and na.finished_at >= $1) auc on true
                     left join lateral ( select count(1) as cnt
                                         from nft_direct_sell na
                                         join events_whitelist ew on na.address = ew.address
                                         where n.address = na.nft
                                           and na.state = 'filled'
                                           and na.finished_at >= $1) ds on true
                     left join lateral ( select count(1) as cnt
                                         from nft_direct_buy na
                                         join events_whitelist ew on na.address = ew.address
                                         where n.address = na.nft
                                           and na.state = 'filled'
                                           and na.finished_at >= $1) db on true
                where n.updated >= $1
                and auc.cnt + ds.cnt + db.cnt > 0
                order by auc.cnt + ds.cnt + db.cnt desc, n.updated desc, n.address desc
                limit $2 offset $3
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
                    and (($4::bool and n.forsale is not null and n."forsale_status: _" = 'active' and exists (
                        select 1 from nft_direct_sell nds where nds.nft = n.address and nds.created <= now() and nds.state = 'active'
                    )) or (not $4::bool and n.forsale is null)
                    or ($3::bool and n.auction is not null and n."auction_status: _" = 'active') or (not $3::bool and n.auction is null))
                )
                or (
                    $3::bool is null
                    and (($4::bool and n.forsale is not null and n."forsale_status: _" = 'active'  and exists (
                        select 1 from nft_direct_sell nds where nds.nft = n.address and nds.created <= now() and nds.state = 'active'
                    )) or (not $4::bool and n.forsale is null))
                )
                or (
                    $4::bool is null
                    and (($3::bool and n.auction is not null and n."auction_status: _" = 'active' ) or (not $3::bool and n.auction is null))
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
                        let _ = write!(sql, "order by n.{field}, n.name");
                    }
                    OrderDirection::Desc => {
                        let _ = write!(sql, "order by coalesce(n.{field}, 0) desc, n.name desc");
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

    pub async fn nft_random_buy(
        &self,
        max_price: i64,
        limit: i32,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as(
            r#"
                SELECT n.*,
                n."auction_status: _" as auction_status,
                n."forsale_status: _" as forsale_status,
                0::int8 total_count
                FROM nft_details n
                INNER JOIN nft_collection c ON n.collection = c.address
                WHERE n.burned = false
                and c.verified = true and n."forsale_status: _" = 'active'
                and c.created <= now()
                and n.floor_price <= $1
                limit $2
            "#,
        )
        .bind(max_price)
        .bind(limit)
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn get_nft_auction(&self, address: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
        SELECT a.*, count(1) over () as "cnt!" FROM nft_auction_search a WHERE a.address = $1"#,
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

    pub async fn get_nft_auction_by_nft(&self, nft: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
                SELECT a.*, count(1) over () as "cnt!" FROM nft_auction_search a
                WHERE a.nft = $1 and a."status: _" in ('active', 'expired')
                order by a.created_at DESC limit 1"#,
            nft
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn get_nft_auction_last_bid(
        &self,
        auction: &String,
    ) -> sqlx::Result<Option<NftAuctionBid>> {
        sqlx::query_as!(
            NftAuctionBid,
            r#"
            SELECT
                auction as "auction!",
                buyer as "buyer!",
                price as "price!",
                usd_price as "usd_price",
                created_at as "created_at!",
                next_bid_value as "next_bid_value!",
                next_bid_usd_value as "next_bid_usd_value",
                tx_lt as "tx_lt!",
                active as "active!",
                 count (1) over () as "cnt!"
            FROM nft_auction_bids_view
            WHERE auction = $1 AND active is true
            LIMIT 1
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
            "
        SELECT
            auction as \"auction!\",
            buyer as \"buyer!\",
            price as \"price!\",
            usd_price as \"usd_price\",
            created_at as \"created_at!\",
            next_bid_value as \"next_bid_value!\",
            next_bid_usd_value as \"next_bid_usd_value\",
            tx_lt as \"tx_lt!\",
            active as \"active!\",
            count (1) over () as \"cnt!\"
        FROM nft_auction_bids_view
        WHERE auction = $1
        ORDER BY created_at DESC LIMIT $2 OFFSET $3",
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
        match sort {
            AuctionsSortOrder::BidsCount => {
                sqlx::query_as!(
                    NftAuction,
                    r#"
                        SELECT a.*, count(1) over () as "cnt!"
                        FROM nft_auction_search a
                        INNER JOIN nft n ON a.nft = n.address
                        WHERE
                        (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                        AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                        AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                        ORDER BY a.bids_count ASC
                        LIMIT $4 OFFSET $5"#,
                    owners,
                    collections,
                    tokens,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(self.db.as_ref())
                .await
            }
            AuctionsSortOrder::StartDate => {
                sqlx::query_as!(
                    NftAuction,
                    r#"
                        SELECT a.*, count(1) over () as "cnt!"
                        FROM nft_auction_search a
                        INNER JOIN nft n ON a.nft = n.address
                        WHERE
                        (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                        AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                        AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                        ORDER BY a.created_at DESC
                        LIMIT $4 OFFSET $5"#,
                    owners,
                    collections,
                    tokens,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(self.db.as_ref())
                .await
            }
            _ => {
                sqlx::query_as!(
                    NftAuction,
                    r#"
                SELECT a.*,  count (1) over () as "cnt!"
                FROM nft_auction_search a
                INNER JOIN nft n ON a.nft = n.address
                WHERE
                (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                ORDER BY a.created_at DESC
                LIMIT $4 OFFSET $5"#,
                    owners,
                    collections,
                    tokens,
                    limit as i64,
                    offset as i64
                )
                .fetch_all(self.db.as_ref())
                .await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_events(
        &self,
        nft: Option<&String>,
        collections: &[String],
        owner: Option<&String>,
        event_type: &[NftEventType],
        category: &[NftEventCategory],
        offset: usize,
        limit: usize,
        with_count: bool,
        verified: Option<bool>,
    ) -> sqlx::Result<NftEventsRecord> {
        let event_types_slice = &event_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];
        let categories_slice = &category
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];

        sqlx::query_file_as!(
            NftEventsRecord,
            "src/db/sql/list_activities.sql",
            categories_slice,
            event_types_slice,
            owner,
            nft,
            collections,
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
        WHERE s.nft = $1
        and s.state = 'active'
        AND (array_length($2::varchar[], 1) is null OR s.state::varchar = ANY($2))
        ORDER BY s.updated DESC LIMIT $3 OFFSET $4
        "#, nft, &status_str, limit as i64, offset as i64)
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
        sqlx::query_as!(NftDirectSell, r#"
        SELECT s.address as "address!", s.created as "created!", s.updated as "updated!", s.tx_lt as "tx_lt!",
        s.nft as "nft!", s.collection,
        s.seller,
        s.price_token as "price_token!", s.price as "price!",
        s.usd_price,
        s.finished_at, s.expired_at,
        s.state as "state!: _",
         count (1) over () as "cnt!",
        s.fee_numerator,
        s.fee_denominator
        FROM nft_direct_sell_usd s
        WHERE s.seller = $1
            AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
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
        SELECT
        x.auction as "auction!",
        x.buyer as "buyer!",
        x.price as "price!",
        x.price_token,
        x.created_at as "created_at!",
        x.next_bid_value,
        x.tx_lt,
        x.active,
        x.usd_price,
        x.next_bid_usd_value,
        x.nft,
        x.collection,
         count (1) over () as "cnt!"
        FROM nft_auction_bids_view x
        WHERE x.buyer = $1
        AND (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND ($3::bool is null OR $3::bool = false OR x.active is true)
        ORDER BY x.created_at DESC LIMIT $4 OFFSET $5
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
            SELECT
            x.auction as "auction!",
            x.buyer as "buyer!",
            x.price as "price!",
            x.price_token,
            x.created_at as "created_at!",
            x.next_bid_value,
            x.tx_lt,
            x.active,
            x.usd_price,
            x.next_bid_usd_value,
            x.nft,
            x.collection,
             count (1) over () as "cnt!"
            FROM nft_auction_bids_view x
                join nft n on x.nft = n.address
                and (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            WHERE x.owner = $1
                and (x.active = true OR ($3::bool IS NULL OR $3::bool = false))
            ORDER BY x.created_at DESC LIMIT $4 OFFSET $5
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
                 select ag.dt as "ts!", (price * tup.usd_price) as "usd_price!"
                 from (
                 select t.finished_at dt, t.price_token, t.price
                 from nft_direct_sell t
                 join events_whitelist ew on t.address = ew.address
                 where t.nft = $1
                   and t.finished_at between $2 and $3
                   and t.state = 'filled'
                 union all
                 select t.finished_at, t.price_token, t.price
                 from nft_direct_buy t
                 join events_whitelist ew on t.address = ew.address
                 where t.nft = $1
                   and t.finished_at between $2 and $3
                   and t.state = 'filled'
                 union all
                 select t.finished_at, t.price_token, t.max_bid
                 from nft_auction t
                 join events_whitelist ew on t.address = ew.address
                 where t.nft = $1
                   and t.finished_at between $2 and $3
                   and t.status = 'completed') as ag
                 join token_usd_prices tup
                      on tup.token = ag.price_token
                      where (price * tup.usd_price) is not null
                 order by 1
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
            "
        select a.collection, a.trait_type, jsonb_agg(a.value) as values
        from (
            select distinct a.collection, a.trait_type, a.value
            from nft_attributes a
            where a.collection is not null and a.trait_type is not null
            order by a.collection ASC, a.trait_type ASC, a.value ASC
        ) as a
        group by a.collection, a.trait_type
        "
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
            r#"select
       case
                   when fee.fee_numerator is not null and fee.fee_denominator is not null then fee.fee_numerator
                   else (ne.args -> 'fee_numerator')::int end "fee_numerator!",
               case
                   when fee.fee_numerator is not null and fee.fee_denominator is not null then fee.fee_denominator
                   else (ne.args -> 'fee_denominator')::int end "fee_denominator!",
               fee.collection,
               fee.nft
                from nft_events ne
                         join roots r
                              on ne.address = r.event_whitelist_address
                                  and r.code = $2::t_root_types
                         left join lateral (
                    select nc.fee_numerator, nc.fee_denominator, max(n.collection) collection, max(e.args ->> 'id') nft
                    from nft n
                        left join nft_events e
                        on e.nft = n.address
                    and e.event_type = 'nft_created'
                             join nft_collection nc on n.collection = nc.address
                        and nc.fee_numerator is not null and nc.fee_denominator is not null
                    where n.owner = $1
                    group by nc.fee_numerator, nc.fee_denominator
                    order by min(nc.fee_numerator / nc.fee_denominator)
                    limit 1

                    ) as fee on true
                where ne.event_type = 'market_fee_default_changed'
                order by created_at desc, created_lt desc, id desc
                limit 1"#,
            owner as &Address,
            root_code as &RootType
        )
            .fetch_one(self.db.as_ref())
            .await
    }
}
