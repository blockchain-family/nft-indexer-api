use super::*;
use crate::handlers::AttributeFilter;
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
        let is_address = sqlx::query_as!(
            SearchResult,
            r#"SELECT
                s.address,
                s.name,
                s.typ as "typ: _",
                s.nft,
                s.collection,
                CASE WHEN m.meta is not null THEN m.meta::jsonb->'preview'->>'source'
                        WHEN s.collection is not null THEN c.logo
                        ELSE null
                END as "image"
            FROM search_index s
            LEFT JOIN nft n ON n.address = s.nft
            LEFT JOIN nft_metadata m ON m.nft = n.address
            LEFT JOIN nft_collection c ON c.address = s.collection
            WHERE s.address = $1
            "#,
            search_str
        )
        .fetch_optional(self.db.as_ref())
        .await?;
        if let Some(r) = is_address {
            return Ok(vec![r]);
        }

        sqlx::query_as!(
            SearchResult,
            r#"SELECT
            s.address,
            s.name,
            s.typ as "typ: _",
            s.nft,
            s.collection,
            CASE WHEN m.meta is not null THEN m.meta::jsonb->'preview'->>'source'
                    WHEN s.collection is not null THEN c.logo
                    ELSE null
            END as "image"
        FROM search_index s
        LEFT JOIN nft n ON n.address = s.nft
        LEFT JOIN nft_metadata m ON m.nft = n.address
        LEFT JOIN nft_collection c ON c.address = s.collection
        WHERE s.search @@ websearch_to_tsquery($1)
        ORDER BY ts_rank_cd(s.search, websearch_to_tsquery($1), 32)
        LIMIT 100"#,
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
         count (1) over () as "cnt!"
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
         count (1) over () as "cnt!"
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
        1::bigint as "cnt!"
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
                SELECT c.*, nft.count as "nft_count!",
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
            1::bigint as "cnt!"
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
             count (1) over () as "cnt!"
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
                SELECT c.*, nft.count as "nft_count!",
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

    pub async fn list_collections(
        &self,
        name: Option<&String>,
        owners: &[String],
        verified: Option<&bool>,
        collections: &[Address],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollectionDetails>> {
        sqlx::query_as!(
            NftCollectionDetails,
            r#"
                SELECT c.*,
                       count(1) over ()  as "cnt!",
                       previews.previews as "previews!"
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
                ORDER BY c.owners_count DESC
                LIMIT $1 OFFSET $2
             "#,
            limit as i64,
            offset as i64,
            owners,
            verified,
            name,
            collections
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
    ) -> sqlx::Result<Vec<NftDetails>> {
        let mut sql = r#"
            SELECT DISTINCT n.*,
            n."auction_status: _" as auction_status,
            n."forsale_status: _" as forsale_status,
            count(1) over () as total_count
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
            // TODO need index trim(na.value #>> '{}')
            let _ = write!(
                sql,
                r#" and exists(
                select 1 from nft_attributes na
                where
                    na.nft = n.address and (lower(na.trait_type) = lower('{0}') and lower(trim(na.value #>> '{{}}')) = lower('{1}'))
                )
            "#,
                attribute.trait_type, attribute.trait_value
            );
        }

        let _ = write!(
            sql,
            r#"
                ORDER BY n.name, n.address
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

    pub async fn list_events(
        &self,
        nft: Option<&String>,
        collections: &[String],
        owner: Option<&String>,
        event_type: &[NftEventType],
        category: &[NftEventCategory],
        offset: usize,
        limit: usize,
    ) -> sqlx::Result<NftEventsRecord> {
        let event_types_slice = &event_type
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];
        let categories_slice = &category
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()[..];
        let events = sqlx::query_file_as!(
            NftEventsRecord,
            "src/db/sql/list_activities.sql",
            categories_slice,
            event_types_slice,
            owner,
            nft,
            collections,
            limit as i64,
            offset as i64
        )
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(events)
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
        count(1) over () as "cnt!"
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
         count (1) over () as "cnt!"
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
        count(1) over () as "cnt!"
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
        count(1) over () as "cnt!"
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

    pub async fn list_nft_price_history_hours(
        &self,
        nft: &String,
        from: Option<usize>,
        to: Option<usize>,
    ) -> sqlx::Result<Vec<NftPrice>> {
        let from = from.map(|x| NaiveDateTime::from_timestamp(x as i64, 0));
        let to = to.map(|x| NaiveDateTime::from_timestamp(x as i64, 0));
        sqlx::query_as!(
            NftPrice,
            "
        SELECT
            date_trunc('hour', p.ts) as ts,
            AVG(p.usd_price) as usd_price,
            count(*) as count
        FROM nft_price_history_usd p
        WHERE p.nft = $1 AND p.price_token is not null
        AND ($2::timestamp is null or p.ts >= $2::timestamp)
        AND ($3::timestamp is null or p.ts < $3::timestamp)
        GROUP BY 1
        ORDER BY 1 ASC
        ",
            Some(nft),
            from,
            to
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_nft_price_history_days(
        &self,
        nft: &String,
        from: Option<usize>,
        to: Option<usize>,
    ) -> sqlx::Result<Vec<NftPrice>> {
        let from = from.map(|x| NaiveDateTime::from_timestamp(x as i64, 0));
        let to = to.map(|x| NaiveDateTime::from_timestamp(x as i64, 0));
        sqlx::query_as!(
            NftPrice,
            "
        SELECT
            date_trunc('day', p.ts) as ts,
            AVG(p.usd_price) as usd_price,
            count(*) as count
        FROM nft_price_history_usd p
        WHERE p.nft = $1 AND p.price_token is not null
        AND ($2::timestamp is null or p.ts >= $2::timestamp)
        AND ($3::timestamp is null or p.ts < $3::timestamp)
        GROUP BY 1
        ORDER BY 1 ASC
        ",
            Some(nft),
            from,
            to
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
}
