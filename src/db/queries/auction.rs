use crate::db::queries::Queries;

use super::*;

use crate::handlers::auction::AuctionsSortOrder;
use sqlx::{self};

impl Queries {
    pub async fn collect_auctions(&self, ids: &[String]) -> sqlx::Result<Vec<NftAuction>> {
        sqlx::query_as_unchecked!(
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
                                   "status: _",
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
                                   count(1) over () as "cnt!"
                            from a

            "#,
            ids
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn get_nft_auction(&self, address: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
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
                   "status: _",
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
                   count(1) over () as "cnt!"
            from nft_auction_search a
            where a.address = $1
            "#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn get_nft_auction_by_nft(&self, nft: &String) -> sqlx::Result<Option<NftAuction>> {
        sqlx::query_as!(
            NftAuction,
            r#"
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
                   "status: _",
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
                   count(1) over () as "cnt!"
            from nft_auction_search a
            where a.nft = $1
              and a."status: _" in ('active', 'expired')
            order by a.created_at desc
            limit 1
            "#,
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
        match sort {
            AuctionsSortOrder::BidsCount => {
                sqlx::query_as!(
                    NftAuction,
                    r#"
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
                           "status: _",
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
                           count(1) over () as "cnt!"
                    from nft_auction_search a
                    where (a.nft_owner = any ($1) or array_length($1::varchar[], 1) is null)
                      and (a.collection = any ($2) or array_length($2::varchar[], 1) is null)
                      and (a.nft = any ($3) or array_length($3::varchar[], 1) is null)
                    order by a.bids_count
                    limit $4 offset $5
                    "#,
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
                           "status: _",
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
                           count(1) over () as "cnt!"
                    from nft_auction_search a
                    where (a.nft_owner = any ($1) or array_length($1::varchar[], 1) is null)
                      and (a.collection = any ($2) or array_length($2::varchar[], 1) is null)
                      and (a.nft = any ($3) or array_length($3::varchar[], 1) is null)
                    order by a.created_at desc
                    limit $4 offset $5
                    "#,
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
                           "status: _",
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
                           count(1) over () as "cnt!"
                    from nft_auction_search a
                    where (a.nft_owner = any ($1) or array_length($1::varchar[], 1) is null)
                      and (a.collection = any ($2) or array_length($2::varchar[], 1) is null)
                      and (a.nft = any ($3) or array_length($3::varchar[], 1) is null)
                    order by a.created_at desc
                    limit $4 offset $5
                    "#,
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
                                           x.collection,
                                           count(1) over ()                        as "cnt!"
                                    from nft_auction_bid x
                                             join offers_whitelist ow on ow.address = x.auction
                                             left join token_usd_prices tup on tup.token = x.price_token
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
                   "cnt!"
            from bids_detailed b
            where b."buyer!" = $1
              and (b.collection = any ($2) or array_length($2::varchar[], 1) is null)
              and ($3::bool is null or $3::bool = false or b.active is true)
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
                                           x.collection,
                                           count(1) over ()                        as "cnt!"
                                    from nft_auction_bid x
                                             left join token_usd_prices tup on tup.token = x.price_token
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
                   "cnt!"
            from bids_detailed x
            where x.owner = $1
              and (x.collection = any ($2) or array_length($2::varchar[], 1) is null)
              and (x.active = true or ($3::bool is null or $3::bool = false))
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
}
