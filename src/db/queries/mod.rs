mod auction;
mod collection;
mod collection_custom;
mod direct_buy;
mod direct_sell;
mod event;
mod nft;
mod user;

use std::sync::Arc;

use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;
use sqlx::{self};

use super::*;
use crate::token::{Token, TokenDict};

#[derive(Debug, Clone)]
pub struct Queries {
    db: Arc<PgPool>,
    wrapped_coin_address: String,
    pub tokens: TokenDict,
}

impl Queries {
    pub fn new(db: Arc<PgPool>, wrapped_coin_address: String, tokens: TokenDict) -> Self {
        Self {
            db,
            wrapped_coin_address,
            tokens,
        }
    }

    fn get_wrapped_coin(&self) -> Token {
        self.tokens
            .get(&self.wrapped_coin_address)
            .expect("missing main token")
            .clone()
    }

    pub async fn update_token_usd_prices(
        &self,
        mut prices: Vec<TokenUsdPrice>,
    ) -> sqlx::Result<()> {
        for price in prices.drain(..) {
            sqlx::query!(
                r#"
                insert into token_usd_prices (token, usd_price, ts)
                values ($1::varchar, $2, $3)
                on conflict (token) do update set usd_price = EXCLUDED.usd_price,
                                                  ts        = EXCLUDED.ts;
                "#,
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
