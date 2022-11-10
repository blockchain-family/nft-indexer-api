use sqlx::{self, postgres::PgPool};
use std::sync::Arc;
use crate::{handlers::AuctionsSortOrder, token::TokenDict};
use super::*;


#[derive(Debug, Clone)]
pub struct Queries {
    db: Arc<PgPool>,
    pub tokens: TokenDict,
}

impl Queries {
    pub fn new(db: Arc<PgPool>, tokens: TokenDict) -> Self {
        Self { db, tokens }
    }

    pub async fn get_nft_details(&self, address: &String) -> sqlx::Result<Option<NftDetails>> {
        sqlx::query_as!(NftDetails, "
        SELECT *
        FROM nft_details
        WHERE address = $1
        ", address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_nft_meta(&self, address: &String) -> sqlx::Result<Option<NftMeta>> {
        sqlx::query_as!(NftMeta, "SELECT * FROM nft_metadata WHERE nft_metadata.nft = $1", address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_collection(&self, address: &String) -> sqlx::Result<Option<NftCollection>> {
        sqlx::query_as!(NftCollection, "SELECT c.*, count(n.*) as nft_count
        FROM nft_collection c
        LEFT JOIN nft n ON n.collection = c.address
        WHERE c.address = $1 GROUP BY 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12", address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_direct_sell(&self, address: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(NftDirectSell, r#"
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
        s.state as "state!: _"
        FROM nft_direct_sell_usd s
        WHERE s.address = $1"#, address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_nft_direct_sell(&self, nft: &String) -> sqlx::Result<Option<NftDirectSell>> {
        sqlx::query_as!(NftDirectSell, r#"
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
        s.state as "state!: _"
        FROM nft_direct_sell_usd s
        WHERE s.nft = $1 and s.state in ('active', 'expired')
        ORDER BY s.created DESC LIMIT 1"#, nft)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_direct_buy(&self, address: &String) -> sqlx::Result<Option<NftDirectBuy>> {
        sqlx::query_as!(NftDirectBuy, r#"
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
        s.state as "state!: _"
        FROM nft_direct_buy_usd s
        WHERE s.address = $1"#, address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn collect_collections(&self, 
        ids: &[String],
    ) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(NftCollection,
            "SELECT c.*, count(n.*) as nft_count
            FROM nft_collection c
            LEFT JOIN nft n ON n.collection = c.address
            WHERE c.address = ANY($1)
            GROUP BY 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12", ids)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_nfts(&self, 
        ids: &[String],
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(NftDetails,
            "SELECT *
            FROM nft_details
            WHERE address = ANY($1)", ids)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_auctions(&self, 
        ids: &[String],
    ) -> sqlx::Result<Vec<NftAuction>> {
        sqlx::query_as!(NftAuction,
            "SELECT * FROM nft_auction_search a WHERE a.address = ANY($1)", ids)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_direct_buy(&self, 
        ids: &[String],
    ) -> sqlx::Result<Vec<NftDirectBuy>> {
        sqlx::query_as!(NftDirectBuy,
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
            s.state as "state!: _"
            FROM nft_direct_buy_usd s
            WHERE s.address = ANY($1)"#, ids)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn collect_direct_sell(&self, 
        ids: &[String],
    ) -> sqlx::Result<Vec<NftDirectSell>> {
        sqlx::query_as!(NftDirectSell,
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
            s.state as "state!: _"
            FROM nft_direct_sell_usd s
            WHERE s.address = ANY($1)"#, ids)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_collections_by_owner(&self, 
        owner: &String,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(NftCollection,
            "SELECT c.*, count(n.*) as nft_count
            FROM nft_collection c
            LEFT JOIN nft n ON n.collection = c.address
            WHERE c.owner = $1
            GROUP BY 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
            LIMIT $2 OFFSET $3"
            , owner, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_collections_by_owner_count(&self, 
        owner: &String,
    ) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT count(*) FROM nft_collection WHERE owner = $1"
            , owner)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }


    pub async fn list_collections(&self,
        name: Option<&String>,
        owners: &[String],
        verified: Option<&bool>,
        collections: &[Address],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(NftCollection, "
        SELECT c.*, count(n.*) as nft_count
        FROM nft_collection c
        LEFT JOIN nft n ON n.collection = c.address
        WHERE (c.owner = ANY($3) OR array_length($3::varchar[], 1) is null)
            AND ($4::boolean is null OR c.verified = $4)
            AND ($5::varchar is null OR c.name LIKE $5)
            AND (c.address = ANY($6) OR array_length($6::varchar[], 1) is null)
        GROUP BY 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
        ORDER BY c.owners_count DESC
        LIMIT $1 OFFSET $2
        ", limit as i64, offset as i64, owners, verified, name, collections)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_collections_count(&self, 
        name: Option<&String>,
        owners: &[String],
        verified: Option<&bool>,
        collections: &[Address],
    ) -> sqlx::Result<i64> {
        sqlx::query!("
        SELECT count(*)
        FROM nft_collection
        WHERE (owner = ANY($1) OR array_length($1::varchar[], 1) is null)
        AND ($2::boolean is null OR verified = $2)
        AND ($3::varchar is null OR name LIKE $3)
        AND (address = ANY($4) OR array_length($4::varchar[], 1) is null)
        ", owners, verified, name, collections)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn nft_search(&self,
        owners: &[Address],
        collections: &[Address],
        _price_from: Option<u64>,
        _price_to: Option<u64>,
        _price_token: Option<Address>,
        forsale: Option<bool>,
        auction: Option<bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(NftDetails, "
        SELECT n.*
        FROM nft_details n
        WHERE
        (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
        and (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
        and (($3::bool is null and $4::bool is null)
            or ($3::bool is not null and $4::bool is not null 
                and (($4::bool and n.forsale is not null and n.\"forsale_status: _\" = 'active') or (not $4::bool and n.forsale is null)
                or ($3::bool and n.auction is not null and n.\"auction_status: _\" = 'active') or (not $3::bool and n.auction is null))
            )
            or (
                $3::bool is null 
                and (($4::bool and n.forsale is not null and n.\"forsale_status: _\" = 'active') or (not $4::bool and n.forsale is null))
            )
            or (
                $4::bool is null
                and (($3::bool and n.auction is not null and n.\"auction_status: _\" = 'active') or (not $3::bool and n.auction is null))
            )
        )
        ORDER BY n.collection
        LIMIT $5 OFFSET $6
        ", owners, collections, auction, forsale, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn nft_search_count(&self, 
        owners: &[Address],
        collections: &[Address],
        _price_from: Option<u64>,
        _price_to: Option<u64>,
        _price_token: Option<Address>,
        forsale: Option<bool>,
        auction: Option<bool>,
    ) -> sqlx::Result<i64> {
        sqlx::query!("
        SELECT count(n.*)
        FROM nft_details n
        WHERE
        (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
        and (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
        and (($3::bool is null and $4::bool is null)
            or ($3::bool is not null and $4::bool is not null 
                and (($4::bool and n.forsale is not null and n.\"forsale_status: _\" = 'active') or (not $4::bool and n.forsale is null)
                or ($3::bool and n.auction is not null and n.\"auction_status: _\" = 'active') or (not $3::bool and n.auction is null))
            )
            or (
                $3::bool is null 
                and (($4::bool and n.forsale is not null and n.\"forsale_status: _\" = 'active') or (not $4::bool and n.forsale is null))
            )
            or (
                $4::bool is null
                and (($3::bool and n.auction is not null and n.\"auction_status: _\" = 'active') or (not $3::bool and n.auction is null))
            )
        )
        ", owners, collections, auction, forsale)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn get_nft_auction(&self, address: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(NftAuction, "
        SELECT * FROM nft_auction_search WHERE address = $1", address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_nft_auction_by_nft(&self, nft: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(NftAuction, "
        SELECT a.* FROM nft_auction_search a
        WHERE a.nft = $1 and a.\"status: _\" in ('active', 'expired')
        order by a.created_at DESC limit 1", nft)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn get_nft_auction_last_bid(&self, auction: &String) -> sqlx::Result<Option<NftAuctionBid>> {
        sqlx::query_as!(NftAuctionBid, "
        SELECT
            auction as \"auction!\",
            buyer as \"buyer!\",
            price as \"price!\",
            usd_price as \"usd_price\",
            created_at as \"created_at!\",
            next_bid_value as \"next_bid_value!\",
            next_bid_usd_value as \"next_bid_usd_value\",
            tx_lt as \"tx_lt!\",
            active as \"active!\"
        FROM nft_auction_bids_view
        WHERE auction = $1 AND active is true
        LIMIT 1
        ", auction)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn list_nft_auction_bids(&self, auction: &String, limit: usize, offset: usize) -> sqlx::Result<Vec<NftAuctionBid>> {
        sqlx::query_as!(NftAuctionBid, "
        SELECT
            auction as \"auction!\",
            buyer as \"buyer!\",
            price as \"price!\",
            usd_price as \"usd_price\",
            created_at as \"created_at!\",
            next_bid_value as \"next_bid_value!\",
            next_bid_usd_value as \"next_bid_usd_value\",
            tx_lt as \"tx_lt!\",
            active as \"active!\"
        FROM nft_auction_bids_view
        WHERE auction = $1
        ORDER BY created_at DESC LIMIT $2 OFFSET $3", auction, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_nft_auction_bids_count(&self, auction: &String) -> sqlx::Result<i64> {
        sqlx::query!("
        SELECT count(*)
        FROM nft_auction_bids_view
        WHERE auction = $1
        ", auction)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_nft_auctions(&self,
        owners: &[Address],
        collections: &[Address],
        tokens: &[Address],
        sort: &AuctionsSortOrder,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuction>> {
        match sort {
            AuctionsSortOrder::BidsCount => {
                sqlx::query_as!(NftAuction, "
                SELECT a.*
                FROM nft_auction_search a
                INNER JOIN nft n ON a.nft = n.address
                WHERE 
                (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                ORDER BY a.bids_count ASC
                LIMIT $4 OFFSET $5", owners, collections, tokens, limit as i64, offset as i64)
                    .fetch_all(self.db.as_ref())
                    .await
            },
            AuctionsSortOrder::StartDate => {
                sqlx::query_as!(NftAuction, "
                SELECT a.*
                FROM nft_auction_search a
                INNER JOIN nft n ON a.nft = n.address
                WHERE 
                (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                ORDER BY a.created_at DESC
                LIMIT $4 OFFSET $5", owners, collections, tokens, limit as i64, offset as i64)
                    .fetch_all(self.db.as_ref())
                    .await
            },
            _ => {
                sqlx::query_as!(NftAuction, "
                SELECT a.*
                FROM nft_auction_search a
                INNER JOIN nft n ON a.nft = n.address
                WHERE 
                (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
                AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
                ORDER BY a.created_at DESC
                LIMIT $4 OFFSET $5", owners, collections, tokens, limit as i64, offset as i64)
                    .fetch_all(self.db.as_ref())
                    .await
            },
        }
    }

    pub async fn list_nft_auctions_count(&self,
        owners: &[Address],
        collections: &[Address],
        tokens: &[Address],
    ) -> sqlx::Result<i64> {
        sqlx::query!("
    SELECT count(a.*) 
    FROM nft_auction_search a
    INNER JOIN nft n ON a.nft = n.address
    WHERE 
    (n.owner = ANY($1) OR array_length($1::varchar[], 1) is null)
    AND (n.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
    AND (a.nft = ANY($3) OR array_length($3::varchar[], 1) is null)
        ", owners, collections, tokens)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_events(&self,
        nft: Option<&String>,
        collection: Option<&String>,
        _owner: Option<&String>,
        typ: &[EventType],
        offset: usize,
        limit: usize,
    ) -> sqlx::Result<Vec<Event>> {
        let typ_str: Vec<String> = typ.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(Event, "
        SELECT e.id, e.address, e.created_at, e.created_lt, e.args,
        e.event_cat as \"event_cat: _\",
        e.event_type as \"event_type: _\"
        FROM nft_events e
        WHERE
            ($3::varchar is null OR e.nft = $3)
            AND ($4::varchar is null OR e.collection = $4)
            AND (array_length($5::varchar[], 1) is null OR e.event_type::varchar = ANY($5))
        LIMIT $1 OFFSET $2
        ", limit as i64, offset as i64,
            nft, collection, &typ_str)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_events_count(&self,
        nft: Option<&String>,
        collection: Option<&String>,
        _owner: Option<&String>,
        typ: &[EventType],
    ) -> sqlx::Result<i64> {
        let typ_str: Vec<String> = typ.iter().map(|x| x.to_string()).collect();
        sqlx::query!("
        SELECT count(*)
        FROM nft_events e
        WHERE
            ($1::varchar is null OR e.nft = $1)
            AND ($2::varchar is null OR e.collection = $2)
            AND (array_length($3::varchar[], 1) is null OR e.event_type::varchar = ANY($3))
        ", nft, collection, &typ_str)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_nft_direct_buy(&self,
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
        s.state as "state!: _"
        FROM nft_direct_buy_usd s
        WHERE s.nft = $1
        AND (array_length($2::varchar[], 1) is null OR s.state::varchar = ANY($2))
        ORDER BY s.updated DESC LIMIT $3 OFFSET $4
        "#, nft, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_nft_direct_buy_count(&self, 
        nft: &String,
        status: &[DirectBuyState],
    ) -> sqlx::Result<i64> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "SELECT count(*) FROM nft_direct_buy s
            WHERE s.nft = $1 AND (array_length($2::varchar[], 1) is null OR s.state::varchar = ANY($2))"
            , nft, &status_str)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_owner_direct_sell(&self,
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
        s.state as "state!: _"
        FROM nft_direct_sell_usd s
        WHERE s.seller = $1 
            AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_sell_count(&self, 
        owner: &String,
        collections: &[String],
        status: &[DirectSellState],
    ) -> sqlx::Result<i64> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "SELECT count(*)
            FROM nft_direct_sell_usd s
            WHERE s.seller = $1 
                AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
            ", owner, collections, &status_str)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_owner_direct_buy(&self,
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
        s.state as "state!: _"
        FROM nft_direct_buy_usd s
        WHERE s.buyer = $1 
            AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_buy_count(&self, 
        owner: &String,
        collections: &[String],
        status: &[DirectBuyState],
    ) -> sqlx::Result<i64> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "SELECT count(*)
            FROM nft_direct_buy_usd s
            WHERE s.buyer = $1 
                AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
            ", owner, collections, &status_str)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_owner_direct_buy_in(&self,
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
        s.state as "state!: _"
        FROM nft_direct_buy_usd s
        INNER JOIN nft n ON n.address = s.nft
        WHERE n.owner = $1 
            AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
        ORDER BY s.updated DESC LIMIT $4 OFFSET $5
        "#, owner, collections, &status_str, limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_direct_buy_in_count(&self, 
        owner: &String,
        collections: &[String],
        status: &[DirectBuyState],
    ) -> sqlx::Result<i64> {
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query!(
            "SELECT count(*)
            FROM nft_direct_buy_usd s
            INNER JOIN nft n ON n.address = s.nft
            WHERE n.owner = $1 
                AND (s.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND (array_length($3::varchar[], 1) is null OR s.state::varchar = ANY($3))
            ", owner, collections, &status_str)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_owner_auction_bids_out(&self,
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuctionBidExt>> {
        sqlx::query_as!(NftAuctionBidExt, r#"
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
        x.collection
        FROM nft_auction_bids_view x
        WHERE x.buyer = $1 
        AND (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND ($3::bool is null OR $3::bool = false OR x.active is true)
        ORDER BY x.created_at DESC LIMIT $4 OFFSET $5
        "#, owner, collections, lastbid.clone(), limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_auction_bids_out_count(&self, 
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
    ) -> sqlx::Result<i64> {
        let lastbid = lastbid.clone();
        sqlx::query!("
            SELECT count(x.*) FROM nft_auction_bids_view x
            WHERE x.buyer = $1 
            AND (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND ($3::bool is null OR $3::bool = false OR x.active is true)
            ", owner, collections, lastbid)
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_owner_auction_bids_in(&self,
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftAuctionBidExt>> {
        sqlx::query_as!(NftAuctionBidExt, r#"
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
        x.collection
        FROM nft_auction_bids_view x
        WHERE x.owner = $1 
            AND (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
            AND ($3::bool is null OR $3::bool = false OR x.active is true)
        ORDER BY x.created_at DESC LIMIT $4 OFFSET $5
        "#, owner, collections, lastbid.clone(), limit as i64, offset as i64)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_owner_auction_bids_in_count(&self, 
        owner: &String,
        collections: &[String],
        lastbid: &Option<bool>,
    ) -> sqlx::Result<i64> {
        sqlx::query!(
            "SELECT count(*)
            FROM nft_auction_bids_view x
            WHERE x.owner = $1 
                AND (x.collection = ANY($2) OR array_length($2::varchar[], 1) is null)
                AND ($3::bool is null OR $3::bool = false OR x.active is true)
            ", owner, collections, lastbid.clone())
            .fetch_one(self.db.as_ref())
            .await
            .map(|r| r.count.unwrap_or_default())
    }

    pub async fn list_nft_price_history_hours(&self,
        nft: &String,
    ) -> sqlx::Result<Vec<NftPrice>> {
        sqlx::query_as!(NftPrice, "
        SELECT
            date_trunc('hour', p.ts) as ts,
            p.price_token,
            AVG(p.price) as price,
            count(*) as count
        FROM nft_price_history p
        WHERE p.nft = $1 AND p.price_token is not null
        GROUP BY 1, 2
        ORDER BY 1 ASC
        ", Some(nft))
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_nft_price_history_days(&self,
        nft: &String,
    ) -> sqlx::Result<Vec<NftPrice>> {
        sqlx::query_as!(NftPrice, "
        SELECT
            date_trunc('day', p.ts) as ts,
            p.price_token,
            AVG(p.price) as price,
            count(*) as count
        FROM nft_price_history p
        WHERE p.nft = $1 AND p.price_token is not null
        GROUP BY 1, 2
        ORDER BY 1 ASC
        ", Some(nft))
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn nft_attributes_dictionary(&self) -> sqlx::Result<Vec<TraitDef>> {
        sqlx::query_as!(TraitDef, "
        select a.collection, a.trait_type, jsonb_agg(a.value) as values
        from (
            select distinct a.collection, a.trait_type, a.value
            from nft_attributes a
            where a.collection is not null and a.trait_type is not null
            order by a.collection ASC, a.trait_type ASC, a.value ASC
        ) as a
        group by a.collection, a.trait_type
        ")
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_attributes_search(&self,
        collection: &String,
        trait_type: &String,
        values: &[serde_json::Value],
    ) -> sqlx::Result<Vec<String>> {
        sqlx::query!("
        select distinct a.nft
        from nft_attributes a
        where a.collection = $1
        and a.trait_type = $2
        and a.value = ANY($3::jsonb[])
        order by 1 asc",
        collection, trait_type, values)
        .fetch_all(self.db.as_ref())
        .await
        .map(|x| x.iter().map(|y| y.nft.clone()).collect())
    }

    pub async fn nft_attributes_search_count(&self,
        collection: &String,
        trait_type: &String,
        values: &[serde_json::Value],
    ) -> sqlx::Result<i64> {
        sqlx::query!("
        select count(distinct a.nft) as count
        from nft_attributes a
        where a.collection = $1
        and a.trait_type = $2
        and a.value = ANY($3::jsonb[])",
        collection, trait_type, values)
        .fetch_one(self.db.as_ref())
        .await
        .map(|r| r.count.unwrap_or_default())
    }

    pub async fn update_token_usd_prices(&self,
        mut prices: Vec<TokenUsdPrice>,
    ) -> sqlx::Result<()> {
        for price in prices.drain(..) {
            sqlx::query!("
            INSERT INTO token_usd_prices (token, usd_price, ts)
            VALUES ($1::varchar, $2, $3) 
            ON CONFLICT (token) DO UPDATE 
            SET
                usd_price = EXCLUDED.usd_price,
                ts = EXCLUDED.ts;
            ", price.token, price.usd_price, price.ts)
            .execute(self.db.as_ref())
            .await?;
        }
        Ok(())
    }
}

