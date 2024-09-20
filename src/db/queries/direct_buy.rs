use crate::db::NftDirectBuy;

use super::*;

use sqlx::{self};

impl Queries {
    pub async fn get_direct_buy(&self, address: &String) -> sqlx::Result<Option<NftDirectBuy>> {
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
                   1::bigint             as "cnt!",
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
            where s.address = $1
            "#,
            address
        )
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn collect_direct_buy(&self, ids: &[String]) -> sqlx::Result<Vec<NftDirectBuy>> {
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
                   1::bigint             as "cnt!",
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
            where s.address = any ($1)
            "#,
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
            where s.buyer = $1
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

    pub async fn list_owner_direct_buy_in(
        &self,
        owner: &String,
        collections: &[String],
        status: &[DirectBuyState],
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftDirectBuy>> {
        sqlx::query_as!(
            NftDirectBuy,
            r#"
            with actual_direct_buy as (select s.address             as address,
                                              s.created             as created,
                                              s.updated             as updated,
                                              s.tx_lt               as tx_lt,
                                              s.nft                 as nft,
                                              s.collection          as collection,
                                              s.buyer               as buyer,
                                              s.price_token         as price_token,
                                              s.price               as price,
                                              s.price * p.usd_price as usd_price,
                                              s.finished_at         as finished_at,
                                              s.expired_at          as expired_at,
                                              case
                                                  when s.state = 'active'::direct_buy_state and
                                                       to_timestamp(0::double precision) < s.expired_at and
                                                       s.expired_at < now()::timestamp then 'expired'::direct_buy_state
                                                  else s.state end  as state,
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
                                                join nft n on n.address = s.nft
                                       where n.owner = $1
                                         and (n.collection = any ($2) or array_length($2::varchar[], 1) is null)
                                       order by s.updated desc)
            select address          as "address!",
                   created          as "created!",
                   updated          as "updated!",
                   tx_lt            as "tx_lt!",
                   nft              as "nft!",
                   collection       as "collection?",
                   buyer            as "buyer?",
                   price_token      as "price_token!",
                   price            as "price!",
                   usd_price        as "usd_price?",
                   finished_at      as "finished_at?",
                   expired_at       as "expired_at?",
                   state            as "state!: _",
                   count(1) over () as  "cnt!",
                   fee_numerator,
                   fee_denominator
            from actual_direct_buy
            where array_length($3::direct_buy_state[], 1) is null
               or state = any ($3)
            limit $4 offset $5
            "#,
            owner,
            collections,
            status as _,
            limit as i64,
            offset as i64
        )
            .fetch_all(self.db.as_ref())
            .await
    }
}
