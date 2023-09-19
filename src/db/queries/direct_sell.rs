use crate::db::queries::Queries;

use super::*;

use sqlx::{self};

impl Queries {
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
}
