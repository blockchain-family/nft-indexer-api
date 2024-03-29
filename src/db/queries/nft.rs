use crate::db::queries::Queries;
use crate::db::NftDetails;
use chrono::NaiveDateTime;

use super::*;

use crate::handlers::nft::{AttributeFilter, NFTListOrder, NFTListOrderField};

use crate::model::OrderDirection;
use sqlx::{self};

impl Queries {
    pub async fn search_all(&self, search_str: &String) -> sqlx::Result<Vec<SearchResult>> {
        sqlx::query_as!(
            SearchResult,
                        r#"
                        with nft_top as (
                select n.address,
                       n.name                                                                           nft_name,
                       nc.name                                                                          collection_name,
                       'nft'                                                                         as object_type,
                       case when m.meta is not null then m.meta::jsonb -> 'preview' ->> 'source' end as "image",
                       case
                           when lower(n.address) = lower($1) then 10
                           when lower(n.name) = lower($1) then 9
                           when n.name like '' || $1 || ' %' then 7.9
                           when n.name like '% ' || $1 || '' then 7.86
                           when n.name like '%' || $1 || '' then 7.855
                           when n.name like '' || $1 || '%' then 7.85
                           when n.name like '% ' || $1 || ' %' then 7.7
                           when n.name like '%' || $1 || '%' then 7
                           when n.address ilike '%' || $1 || '%' then 5
                           else 1 end                                                                   priority
                from nft_verified_mv n
                         left join nft_metadata m on n.address = m.nft
                         join nft_collection nc on n.collection = nc.address
                where (n.name ilike '%' || $1 || '%' or n.description ilike '%' || $1 || '%' or n.address ilike '%' || $1 || '%')
                  and not n.burned
                order by priority desc
                limit 20
            )

            select ag.address as "address!", nft_name, collection_name, object_type as "object_type!", image
            from (
                     select *
                     from nft_top
                     union all

                     select c.address,
                            null            nft_name,
                            c.name          collection_name,
                            'collection' as object_type,
                            c.logo          "image",
                            case
                                when lower(c.address) = lower($1) then 20
                                when lower(c.name) = lower($1) then 19
                                when c.name like '' || $1 || ' %' then 8.9
                                when c.name like '% ' || $1 || '' then 8.86
                                when c.name like '%' || $1 || '' then 8.855
                                when c.name like '' || $1 || '%' then 8.85

                                when c.name like '% ' || $1 || ' %' then 8.7
                                when c.address ilike '%' || $1 || '%' then 6
                                else 2 end  priority
                     from nft_collection c
                     where (c.name ilike '%' || $1 || '%' or c.description ilike '%' || $1 || '%' or
                            c.address ilike '%' || $1 || '%')
                       and c.verified) ag
            order by ag.priority desc
            limit 20
            "#,
            search_str
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn get_nft_details(&self, address: &String) -> sqlx::Result<Option<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
            with details as ( select n.address,
                                     n.collection,
                                     n.owner,
                                     n.manager,
                                     n.name::text                          as name,
                                     n.description,
                                     n.burned,
                                     n.updated,
                                     n.owner_update_lt                     as tx_lt,
                                     m.meta,
                                     auc.auction,
                                     auc."auction_status: _",
                                     sale.forsale,
                                     sale."forsale_status: _",
                                     ( select distinct on (s.address) first_value(s.address) over w
                                       from nft_direct_buy s
                                                left join token_usd_prices tup on tup.token = s.price_token
                                       where state = 'active'
                                         and nft = n.address
                                       window w as (partition by nft order by s.price * tup.usd_price desc)
                                       limit 1 )                           as best_offer,
                                     least(auc.price_usd, sale.price_usd)  as floor_price_usd,
                                     last_deal.last_price                  as deal_price_usd,
                                     case when least(auc.price_usd, sale.price_usd) = auc.price_usd then auc.min_bid
                                          when least(auc.price_usd, sale.price_usd) = sale.price_usd then sale.price
                                          else null::numeric end           as floor_price,
                                     case when least(auc.price_usd, sale.price_usd) = auc.price_usd
                                              then auc.token::character varying
                                          when least(auc.price_usd, sale.price_usd) = sale.price_usd
                                              then sale.token::character varying
                                          else null::character varying end as floor_price_token,
                                     n.id::text                            as nft_id
                              from nft n
                                       left join lateral ( select nph.price * tup.usd_price as last_price
                                                           from nft_price_history nph
                                                                    join offers_whitelist ow on ow.address = nph.source
                                                                    left join token_usd_prices tup on tup.token = nph.price_token
                                                           where nph.nft = n.address
                                                           order by nph.ts desc
                                                           limit 1 ) last_deal on true
                                       left join lateral ( select a.address                 as auction,
                                                                  case when a.status = 'active' and
                                                                            to_timestamp(0) < a.finished_at and
                                                                            a.finished_at < now() then 'expired'
                                                                       else a.status end    as "auction_status: _",
                                                                  a.min_bid * tup.usd_price as price_usd,
                                                                  tup.token,
                                                                  a.min_bid
                                                           from nft_auction a
                                                                    join offers_whitelist ow on ow.address = a.address
                                                                    left join token_usd_prices tup on tup.token = a.price_token
                                                           where a.nft = n.address
                                                             and a.status in ('active', 'expired')
                                                           limit 1 ) auc on true
                                       left join nft_metadata m on m.nft = n.address
                                       left join lateral ( select s.address                                as forsale,
                                                                  case when s.state = 'active' and
                                                                            to_timestamp(0) < s.expired_at and s.expired_at < now()
                                                                           then 'expired' else s.state end as "forsale_status: _",
                                                                  s.price * tup.usd_price                  as price_usd,
                                                                  s.price,
                                                                  tup.token
                                                           from nft_direct_sell s
                                                                    join offers_whitelist ow on ow.address = s.address
                                                                    left join token_usd_prices tup on tup.token = s.price_token
                                                           where s.nft = n.address
                                                             and s.state in ('active', 'expired')
                                                           limit 1 ) sale on true
                              where not n.burned
                                and n.address = $1 )
            select n.address           as "address?",
                   n.collection        as "collection?",
                   n.owner             as "owner?",
                   n.manager           as "manager?",
                   n.name              as "name?",
                   n.description       as "description?",
                   n.burned            as "burned?",
                   n.updated           as "updated?",
                   n.tx_lt             as "tx_lt?",
                   n.meta              as "meta?",
                   n.auction           as "auction?",
                   n."auction_status: _",
                   n.forsale           as "forsale?",
                   n."forsale_status: _",
                   n.best_offer        as "best_offer?",
                   n.floor_price_usd   as "floor_price_usd?",
                   n.deal_price_usd    as "deal_price_usd?",
                   n.floor_price       as "floor_price?",
                   n.floor_price_token as "floor_price_token?",
                   n.nft_id            as "nft_id?",
                   1::bigint           as "total_count!"
            from details n;
            "#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn collect_nfts(&self, ids: &[String]) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
            select n.*, 1::bigint as "total_count!"
            from nft_details n
            where n.address = any ($1)
            "#,
            ids
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_top_search(
        &self,
        from: NaiveDateTime,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as!(
            NftDetails,
            r#"
                select n.*, count(1) over () as "total_count!"
                from (
                         select *
                         from nft_verified_mv nvm
                                  left join lateral (
                             select count(1) as cnt
                             from nft_price_history nph
                                      join offers_whitelist ow on ow.address = nph.source
                             where nvm.address = nph.nft
                               and nph.ts >= $1
                             ) offers on true
                         where nvm.updated > $1
                           and offers.cnt > 0
                         order by offers.cnt desc, nvm.updated desc, nvm.address desc
                         limit $2 offset $3) ag
                         join nft_details n
                              on ag.address = n.address
            "#,
            from,
            limit,
            offset
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn nft_search(
        &self,
        owners: &[Address],
        collections: &[Address],
        forsale: Option<bool>,
        auction: Option<bool>,
        verified: Option<bool>,
        limit: usize,
        offset: usize,
        _attributes: &[AttributeFilter],
        order: Option<NFTListOrder>,
        with_count: bool,
    ) -> sqlx::Result<Vec<NftDetails>> {
        let sql: &str = include_str!("../sql/nfts.sql");
        let forsale = forsale.unwrap_or(false);
        let auction = auction.unwrap_or(false);

        let mut order_direction_result = "asc".to_string();
        let mut deals_order_field = "ag.name";
        let mut enable_sales_query = false;
        let mut order_result = "".to_string();
        let mut nfts_direction_default = "asc".to_string();

        if let Some(order) = order {
            order_direction_result = order.direction.to_string();
            enable_sales_query = true;
            deals_order_field = match order.field {
                NFTListOrderField::FloorPriceUsd => {
                    match order.direction {
                        OrderDirection::Asc => order_result = "order by coalesce(n.floor_price_usd, least(auc.price_usd, sale.price_usd)) asc, n.name asc, n.address asc".to_string(),
                        OrderDirection::Desc => order_result = "order by coalesce(n.floor_price_usd, coalesce(least(auc.price_usd, sale.price_usd)), 0) desc, n.name asc, n.address asc".to_string()
                    };

                    "coalesce(ag.price_usd, 0)"
                }
                NFTListOrderField::DealPriceUsd => {
                    match order.direction {
                        OrderDirection::Asc => order_result = "order by coalesce(n.floor_price_usd, least(auc.price_usd, sale.price_usd)) asc, n.name asc, n.address asc".to_string(),
                        OrderDirection::Desc => order_result = "order by coalesce(n.floor_price_usd, coalesce(least(auc.price_usd, sale.price_usd)), 0) desc, n.name asc, n.address asc".to_string()
                    };
                    "coalesce(ag.price_usd, 0)"
                } // ???
                NFTListOrderField::Name => {
                    enable_sales_query = false;
                    nfts_direction_default = order_direction_result.clone();
                    "ag.name"
                }
            }
        }

        if forsale || auction {
            enable_sales_query = false;
        }

        let with_optimized: bool = if enable_sales_query {
            sqlx::query_scalar!("select case
                               when not $3::bool then false
                               when $1::text[] = '{}'::text[] and $2::text[] = '{}'::text[] then true
                               when coalesce(array_length($2::text[], 1), 0) > 0 and
                                    coalesce(array_length($1::text[], 1), 0) > 0 and (
                                                                                          select count(1)
                                                                                          from nft n
                                                                                          where n.owner = any ($1::text[])
                                                                                            and n.collection = any ($2::text[])
                                                                                      ) > 1000 then true

                               when coalesce(array_length($2::text[], 1), 0) > 0 and $1::text[] = '{}'::text[] and (select sum(nft_count)
                                                                                                      from nft_collection_details ncd
                                                                                                      where ncd.address = any ($2::text[])
                                                                                                     ) > 1000 then true


                               when $2::text[] = '{}'::text[] and coalesce(array_length($1::text[], 1), 0) > 0 and (
                                                                                                         select count(1)
                                                                                                         from nft_verified_mv n
                                                                                                         where n.owner = any ($1::text[])
                                                                                                     ) > 1000 then true
                               else false
                               end is_enabled",
            owners,
            collections,
            enable_sales_query
        ).fetch_one(self.db.as_ref())
                .await?.expect("failed to get value")
        } else {
            false
        };

        let sql = sql.replace("#ORDER_DIRECTION#", &order_direction_result);
        let sql = sql.replace("#DEALS_ORDER_FIELD#", deals_order_field);
        let sql = sql.replace("#NFTS_DIRECTION_BASE#", &nfts_direction_default);

        let sql = if !verified.unwrap_or(true) {
            sql.replace("nft_verified_mv", "nft")
        } else {
            sql
        };

        let sql = if !with_optimized {
            sql.replace("#ORDER_RESULT#", &order_result)
        } else {
            sql.replace("#ORDER_RESULT#", "")
        };

        sqlx::query_as(&sql)
            .bind(owners)
            .bind(collections)
            .bind(auction)
            .bind(forsale)
            .bind(limit as i64)
            .bind(offset as i64)
            .bind(with_count)
            .bind(with_optimized)
            .fetch_all(self.db.as_ref())
            .await

        // for attribute in attributes {
        //     let values = attribute
        //         .trait_values
        //         .iter()
        //         .enumerate()
        //         .map(|it| format!("'{}'", it.1.to_lowercase()))
        //         .collect::<Vec<String>>()
        //         .join(",");
        //     // TODO need index trim(na.value #>> '{}')
        //     let _ = write!(
        //         sql,
        //         r#" and exists(
        //             select 1 from nft_attributes na
        //             where
        //                 na.nft = n.address and (
        //                     lower(na.trait_type) = lower('{0}') and
        //                     lower(trim(na.value #>> '{{}}')) in ({1})
        //                 )
        //         )
        //     "#,
        //         attribute.trait_type, values
        //     );
        // }
    }

    pub async fn get_traits(&self, nft: &Address) -> sqlx::Result<Vec<NftTraitRecord>> {
        sqlx::query_as!(
            NftTraitRecord,
            r#"
            with nft_attributes as ( select jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' as trait_type,
                                            jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      as trait_value,
                                            nm.meta,
                                            n.collection                                                  as nft_collection,
                                            nm.nft
                                     from nft_metadata nm
                                              join nft n on n.address = nm.nft
                                     where nm.meta -> 'attributes' is not null
                                       and nm.nft = $1 ),
                 nft_attributes_col as ( select jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' as trait_type,
                                                jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      as trait_value,
                                                nm.nft
                                         from nft_metadata nm
                                         where nm.nft in ( select n2.address
                                                           from nft n2
                                                                    join nft n3 on n3.address = $1 and n2.collection = n3.collection ) )
            select (na.trait_type #>> '{}')::text  as trait_type,
                   (na.trait_value #>> '{}')::text as trait_value,
                   count(*)                        as "cnt!"
            from nft_attributes na
                     left join nft_attributes_col na2 on na.trait_type = na2.trait_type and na.trait_value = na2.trait_value
            group by na.trait_type, na.trait_value
            "#,
            nft
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_nft_price_history(
        &self,
        nft: &str,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> sqlx::Result<Vec<NftPrice>> {
        sqlx::query_as!(
            NftPrice,
            r#"
            select ts, usd_price as "usd_price!"
            from nft_price_history nph
                     inner join offers_whitelist ow on ow.address = nph.source
            where nft = $1
              and ts between $2 and $3
              and usd_price is not null
            order by ts
            "#,
            nft,
            from,
            to,
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_random_buy(
        &self,
        max_price: i64,
        limit: i32,
    ) -> sqlx::Result<Vec<NftDetails>> {
        sqlx::query_as(
            r#"
                with deals as (
                    select n.address,
                           n.collection,
                           n.owner,
                           n.manager,
                           n.name,
                           n.description,
                           n.burned,
                           n.updated,
                           n.owner_update_lt,
                           n.id,
                           tup.token               as floor_price_token,
                           s.price                 as floor_price,
                           s.price * tup.usd_price as floor_price_usd,
                           null                    as auction,
                           null::auction_status    as auction_status,
                           s.address               as forsale,
                           s.state                 as forsale_status

                    from nft_direct_sell s
                             join nft_verified_mv n
                                  on n.address = s.nft
                             join offers_whitelist ow on ow.address = s.address
                             left join token_usd_prices tup on tup.token = s.price_token
                    where s.price <= $1
                      and s.state = 'active'::direct_sell_state
                      and (s.expired_at = to_timestamp(0) or s.expired_at > now())
                    order by random()
                    limit $2
                )

                select n.address,
                       n.collection,
                       n.owner,
                       n.manager,
                       n.name::text        as name,
                       n.description,
                       n.burned,
                       n.updated,
                       n.owner_update_lt   as tx_lt,
                       m.meta,
                       n.auction              auction,
                       n.auction_status::auction_status    as auction_status,
                       n.forsale           as forsale,
                       n.forsale_status::direct_sell_state    as forsale_status,
                       best_offer.address  as best_offer,
                       n.floor_price_usd      floor_price_usd,
                       last_deal.last_price   deal_price_usd,
                       n.floor_price       as floor_price,
                       n.floor_price_token as floor_price_token,
                       n.id::text          as nft_id,
                       0::int8                   as total_count
                from deals n
                         left join nft_metadata m on m.nft = n.address
                         left join lateral ( SELECT s.address
                                             FROM nft_direct_buy s
                                                      JOIN offers_whitelist ow ON ow.address = s.address
                                                      JOIN token_usd_prices tup ON tup.token = s.price_token
                                             WHERE s.state = 'active'
                                               AND s.nft = n.address
                                             group by s.address, (s.price * tup.usd_price)
                                             HAVING (s.price * tup.usd_price) = MAX(s.price * tup.usd_price)
                                             LIMIT 1 ) best_offer on true


                         left join lateral ( select nph.price * tup.usd_price as last_price
                                             from nft_price_history nph
                                                      join offers_whitelist ow on ow.address = nph.source
                                                      join token_usd_prices tup on tup.token = nph.price_token
                                             where nph.nft = n.address
                                             order by nph.ts desc
                                             limit 1 ) last_deal on true


            "#
        ).bind(max_price).bind(limit)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn nft_sell_count(&self, max_price: i64) -> sqlx::Result<Option<i64>> {
        sqlx::query_scalar!(
            r#"
            select count(1)
            from nft n
                     join nft_collection c on n.collection = c.address
                     join nft_direct_sell nds on nds.nft = n.address and nds.created <= now() and
                                                 (now() <= nds.expired_at or nds.expired_at = to_timestamp(0)) and
                                                 nds.state = 'active' and nds.price <= $1::int8
                     join offers_whitelist ow on ow.address = nds.address
            where n.burned is false
              and c.verified is true
           "#,
            max_price
        )
            .fetch_one(self.db.as_ref())
            .await
    }

    pub async fn nft_attributes_dictionary(&self) -> sqlx::Result<Vec<TraitDef>> {
        sqlx::query_as!(
            TraitDef,
            r#"
            select a.collection, a.trait_type, jsonb_agg(a.value) as values
            from ( select distinct a.collection, a.trait_type, a.value
                   from nft_attributes a
                   order by a.collection, a.trait_type, a.value ) as a
            group by a.collection, a.trait_type
            "#
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
            r#"
            select distinct a.nft
            from nft_attributes a
            where a.collection = $1
              and a.trait_type = $2
              and a.value = any ($3::jsonb[])
            order by 1 asc
            "#,
            collection,
            trait_type,
            values
        )
        .fetch_all(self.db.as_ref())
        .await
        .map(|x| x.iter().map(|y| y.nft.clone()).collect())
    }
}
