use crate::db::NftDirectBuy;

use super::*;

use sqlx::{self};

impl Queries {
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
}
