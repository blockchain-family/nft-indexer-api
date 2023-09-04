use crate::db::queries::Queries;

use super::*;

use crate::handlers::auction::AuctionsSortOrder;
use sqlx::{self};

impl Queries {
    pub async fn collect_auctions(&self, ids: &[String]) -> sqlx::Result<Vec<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
                r#"SELECT a.*, count(1) over() as "cnt!" FROM nft_auction_search a WHERE a.address = ANY($1)"#,
            ids
        )
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
}
