use crate::db::NftDirectBuy;

use super::*;

use sqlx::{self};

impl Queries {
    pub async fn get_direct_buy(&self, address: &String) -> sqlx::Result<Option<NftDirectBuy>> {
        sqlx::query_as!(
            NftDirectBuy,
            r#"
            select s.address     as "address!",
                   s.created     as "created!",
                   s.updated     as "updated!",
                   s.tx_lt       as "tx_lt!",
                   s.nft         as "nft!",
                   s.collection,
                   s.buyer,
                   s.price_token as "price_token!",
                   s.price       as "price!",
                   s.usd_price,
                   s.finished_at,
                   s.expired_at,
                   s.state       as "state!: _",
                   1::bigint     as "cnt!",
                   s.fee_numerator,
                   s.fee_denominator
            from nft_direct_buy_usd s
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
            select s.address     as "address!",
                   s.created     as "created!",
                   s.updated     as "updated!",
                   s.tx_lt       as "tx_lt!",
                   s.nft         as "nft!",
                   s.collection,
                   s.buyer,
                   s.price_token as "price_token!",
                   s.price       as "price!",
                   s.usd_price,
                   s.finished_at,
                   s.expired_at,
                   s.state       as "state!: _",
                   1::bigint     as "cnt!",
                   s.fee_numerator,
                   s.fee_denominator
            from nft_direct_buy_usd s
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
            select s.address        as "address!",
                   s.created        as "created!",
                   s.updated        as "updated!",
                   s.tx_lt          as "tx_lt!",
                   s.nft            as "nft!",
                   s.collection,
                   s.buyer,
                   s.price_token    as "price_token!",
                   s.price          as "price!",
                   s.usd_price,
                   s.finished_at,
                   s.expired_at,
                   s.state          as "state!: _",
                   count(1) over () as "cnt!",
                   s.fee_numerator,
                   s.fee_denominator
            from nft_direct_buy_usd s
            where s.nft = $1
              and s.state = 'active'
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
            select s.address        as "address!",
                   s.created        as "created!",
                   s.updated        as "updated!",
                   s.tx_lt          as "tx_lt!",
                   s.nft            as "nft!",
                   s.collection,
                   s.buyer,
                   s.price_token    as "price_token!",
                   s.price          as "price!",
                   s.usd_price,
                   s.finished_at,
                   s.expired_at,
                   s.state          as "state!: _",
                   count(1) over () as "cnt!",
                   s.fee_numerator,
                   s.fee_denominator
            from nft_direct_buy_usd s
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
        let status_str: Vec<String> = status.iter().map(|x| x.to_string()).collect();
        sqlx::query_as!(
            NftDirectBuy,
            r#"
            select s.address        as "address!",
                   s.created        as "created!",
                   s.updated        as "updated!",
                   s.tx_lt          as "tx_lt!",
                   s.nft            as "nft!",
                   s.collection,
                   s.buyer,
                   s.price_token    as "price_token!",
                   s.price          as "price!",
                   s.usd_price,
                   s.finished_at,
                   s.expired_at,
                   s.state          as "state!: _",
                   count(1) over () as "cnt!",
                   s.fee_numerator,
                   s.fee_denominator
            from nft_direct_buy_usd s
                     inner join nft n on n.address = s.nft
            where n.owner = $1
              and (n.collection = any ($2) or array_length($2::varchar[], 1) is null)
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
