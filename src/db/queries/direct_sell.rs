use crate::db::queries::Queries;

use super::*;

use sqlx::{self};

impl Queries {
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
}
