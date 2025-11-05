use anyhow::{anyhow, bail};
use chrono::NaiveDateTime;
use sqlx::{self};

use super::*;
use crate::db::queries::Queries;
use crate::db::query_params::nft::NftSearchParams;
use crate::db::{NftDetails, NftMimetype};
use crate::handlers::nft::{AttributeFilter, NFTListOrder, NFTListOrderField};
use crate::handlers::requests::OrderDirection;

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
                    from nft_verified_extended n
                             left join nft_metadata m on n.address = m.nft
                             join nft_collection nc on n.collection = nc.address
                    where n.name ilike '%' || $1 || '%'
                       or lower(n.address) = lower($1)
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
                                     ( select s.address
                                       from nft_direct_buy s
                                                left join token_usd_prices tup on tup.token = s.price_token
                                       where state = 'active'
                                         and nft = n.address
                                       order by s.price * tup.usd_price desc nulls last
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

    pub async fn nft_get_for_banner(&self) -> sqlx::Result<Vec<NftForBanner>> {
        sqlx::query_as!(
            NftForBanner,
            r#"
            select distinct on (result.collection_address)
                result.name as "name!",
                result.collection_address as "collection_address!",
                result.nft_address as "nft_address!",
                result.picture as "picture!",
                result.mimetype "mimetype!"
            from (
                select
                     nc."name",
                     nc.address as "collection_address",
                     n.address as "nft_address",
                     nmd.meta -> 'preview' ->> 'source' as "picture",
                     nmd.meta -> 'preview' ->> 'mimetype' as "mimetype"
                from
                     nft_collection nc
                     join nft n on nc.address = n.collection
                     join nft_metadata nmd on nmd.nft = n.address
                where
                     nmd.meta -> 'preview' ->> 'source' is not null
                     and nc.address in (
                         select nc.address from nft_collection nc where nc.for_banner
                     )
            ) result
            order by result.collection_address, random()
            "#
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn collect_nfts(&self, ids: &[String]) -> sqlx::Result<Vec<NftDetails>> {
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
                                and n.address = any ($1) )
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
            from details n
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
                with nfts as (
                    select nvm.address, count(1) as subcnt, nvm.updated
                    from nft_verified_extended nvm
                             left join nft_price_history nph
                                       on nvm.address = nph.nft
                                           and nph.ts >= $1
                             join offers_whitelist ow on ow.address = nph.source

                    where nvm.updated >= $1
                    group by nvm.address, nvm.updated, nvm.address
                    having count(1) > 0
                    order by count(1) desc, nvm.updated desc, nvm.address desc
                    limit $2 offset $3
                )

                select n.address as "address?",
                       n.collection as "collection?",
                       n.owner as "owner?",
                       n.manager as "manager?",
                       n.name::text                         as "name?",
                       n.description as "description?",
                       n.burned as "burned?",
                       n.updated as "updated?",
                       n.owner_update_lt                    as "tx_lt?",
                       m.meta as "meta?",
                       auc.auction as "auction?",
                       auc."auction_status: _",
                       sale.forsale as "forsale?",
                       sale."forsale_status: _",
                       (select distinct on (s.address) first_value(s.address) over w
                        from nft_direct_buy s
                                 left join token_usd_prices tup on tup.token = s.price_token
                        where state = 'active'
                          and nft = n.address
                            window w as (partition by nft order by s.price * tup.usd_price desc)
                        limit 1)                            as "best_offer?",
                       least(auc.price_usd, sale.price_usd) as "floor_price_usd?",
                       last_deal.last_price                 as "deal_price_usd?",
                       case
                           when least(auc.price_usd, sale.price_usd) = auc.price_usd then auc.min_bid
                           when least(auc.price_usd, sale.price_usd) = sale.price_usd then sale.price
                           else null::numeric end           as "floor_price?",
                       case
                           when least(auc.price_usd, sale.price_usd) = auc.price_usd
                               then auc.token::character varying
                           when least(auc.price_usd, sale.price_usd) = sale.price_usd
                               then sale.token::character varying
                           else null::character varying end as "floor_price_token?",
                       n.id::text                           as "nft_id?",
                       count(1) over ()                          as "total_count!"
                from nft n
                         join nfts
                              on nfts.address = n.address
                         left join lateral ( select nph.price * tup.usd_price as last_price
                                             from nft_price_history nph
                                                      join offers_whitelist ow on ow.address = nph.source
                                                      left join token_usd_prices tup on tup.token = nph.price_token
                                             where nph.nft = n.address
                                             order by nph.ts desc
                                             limit 1 ) last_deal on true
                         left join lateral ( select a.address                 as auction,
                                                    case
                                                        when a.status = 'active' and
                                                             to_timestamp(0) < a.finished_at and
                                                             a.finished_at < now() then 'expired'
                                                        else a.status end     as "auction_status: _",
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
                         left join lateral ( select s.address               as forsale,
                                                    case
                                                        when s.state = 'active' and
                                                             to_timestamp(0) < s.expired_at and s.expired_at < now()
                                                            then 'expired'
                                                        else s.state end    as "forsale_status: _",
                                                    s.price * tup.usd_price as price_usd,
                                                    s.price,
                                                    tup.token
                                             from nft_direct_sell s
                                                      join offers_whitelist ow on ow.address = s.address
                                                      left join token_usd_prices tup on tup.token = s.price_token
                                             where s.nft = n.address
                                               and s.state in ('active', 'expired')
                                             limit 1 ) sale on true
                where not n.burned
                order by nfts.subcnt desc, nfts.updated desc, nfts.address desc
            "#,
            from,
            limit,
            offset
        )
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn nft_get_types(&self, verified: bool) -> sqlx::Result<Vec<NftMimetype>> {
        sqlx::query_as!(
            NftMimetype,
            r#"
            select distinct mimetype as "mimetype!"
            from collection_type_mv
            where verified = $1
                and mimetype is not null
            "#,
            verified
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn nft_search(&self, params: &NftSearchParams<'_>) -> sqlx::Result<Vec<NftDetails>> {
        let sql: &str = include_str!("../sql/nfts_full.sql");

        let (attributes_filter, bind_params) = build_attributes_filter(12, params.attributes, "n")?;

        let order = params.order.clone().unwrap_or(NFTListOrder {
            field: NFTListOrderField::Name,
            direction: OrderDirection::Asc,
        });

        let order = match order.field {
            NFTListOrderField::FloorPriceUsd => match order.direction {
                OrderDirection::Asc => {
                    "order by floor_price_usd.val asc, n.name COLLATE numeric asc, n.address asc"
                }
                OrderDirection::Desc => {
                    "order by coalesce(floor_price_usd.val, 0) desc, n.name COLLATE numeric asc, n.address asc"
                }
            },
            NFTListOrderField::DealPriceUsd => match order.direction {
                OrderDirection::Asc => {
                    "order by floor_price_usd.val asc, n.name COLLATE numeric asc, n.address asc"
                }
                OrderDirection::Desc => {
                    "order by coalesce(floor_price_usd.val, 0) desc, n.name COLLATE numeric asc, n.address asc"
                }
            },
            NFTListOrderField::Name => match order.direction {
                OrderDirection::Asc => "order by n.name COLLATE numeric asc, n.address asc",
                OrderDirection::Desc => "order by n.name COLLATE numeric desc, n.address asc",
            },
        };
        let sql = sql.replace("#ORDER#", order);
        let sql = sql.replace("#ATTRIBUTES#", &attributes_filter);
        let mut db_query = sqlx::query_as(&sql)
            .bind(params.owners)
            .bind(params.collections)
            .bind(params.auction)
            .bind(params.forsale)
            .bind(params.limit as i64)
            .bind(params.offset as i64)
            .bind(params.with_count)
            .bind(params.price_from)
            .bind(params.price_to)
            .bind(params.price_token)
            .bind(params.nft_type);

        for (param1, param2) in bind_params {
            db_query = db_query.bind(param1).bind(param2);
        }

        db_query.fetch_all(self.db.as_ref()).await
    }

    pub async fn nft_search_verified(
        &self,
        params: &NftSearchParams<'_>,
    ) -> sqlx::Result<Vec<NftDetails>> {
        let sql: &str = include_str!("../sql/nfts_verified.sql");

        let (attributes_filter, bind_params) =
            build_attributes_filter(12, params.attributes, "nve")?;

        let order = params.order.clone().unwrap_or(NFTListOrder {
            field: NFTListOrderField::Name,
            direction: OrderDirection::Asc,
        });

        let order = match order.field {
            NFTListOrderField::FloorPriceUsd => match order.direction {
                OrderDirection::Asc => {
                    "order by LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd) asc, nve.name  COLLATE numeric asc, nve.address asc"
                }
                OrderDirection::Desc => {
                    "order by coalesce(LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd), 0) desc, nve.name COLLATE numeric asc, nve.address asc"
                }
            },
            NFTListOrderField::DealPriceUsd => match order.direction {
                OrderDirection::Asc => {
                    "order by LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd) asc, nve.name COLLATE numeric asc, nve.address asc"
                }
                OrderDirection::Desc => {
                    "order by coalesce(LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd) desc, nve.name COLLATE numeric asc, nve.address asc"
                }
            },
            NFTListOrderField::Name => match order.direction {
                OrderDirection::Asc => "order by nve.name COLLATE numeric asc, nve.address asc",
                OrderDirection::Desc => "order by nve.name COLLATE numeric desc, nve.address asc",
            },
        };

        let sql = sql.replace("#ORDER#", order);
        let sql = sql.replace("#ATTRIBUTES#", &attributes_filter);
        let mut db_query = sqlx::query_as(&sql)
            .bind(params.owners)
            .bind(params.collections)
            .bind(params.auction)
            .bind(params.forsale)
            .bind(params.limit as i64)
            .bind(params.offset as i64)
            .bind(params.with_count)
            .bind(params.price_from)
            .bind(params.price_to)
            .bind(params.price_token)
            .bind(params.nft_type);

        for (param1, param2) in bind_params {
            db_query = db_query.bind(param1).bind(param2);
        }

        db_query.fetch_all(self.db.as_ref()).await
    }

    pub async fn get_traits(&self, nft: &Address) -> sqlx::Result<Vec<NftTraitRecord>> {
        sqlx::query_as!(
            NftTraitRecord,
            r#"
            with traits as ( select trait_type, value as trait_value from nft_attributes where nft = $1 )
            select traits.trait_type                                                             as "trait_type?",
                   traits.trait_value                                                            as "trait_value?",
                   ( select count(*)
                     from nft_attributes na
                     where traits.trait_type = na.trait_type and traits.trait_value = na.value
                     and na.collection = (select collection from nft nn where nn.address = $1)
                      ) as "cnt!"
            from traits
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
                           n2.owner,
                           n2.manager,
                           n.name,
                           n2.description,
                           n2.burned,
                           n.updated,
                           n2.owner_update_lt,
                           n2.id,
                           tup.token               as floor_price_token,
                           s.price                 as floor_price,
                           s.price * tup.usd_price as floor_price_usd,
                           null                    as auction,
                           null::auction_status    as auction_status,
                           s.address               as forsale,
                           s.state                 as forsale_status

                    from nft_direct_sell s
                             join nft_verified_extended n
                                  on n.address = s.nft
                             join nft n2
                                on n2.address = n.address
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

    pub async fn nft_price_range_verified(
        &self,
        collections: &[Address],
        attributes: &[AttributeFilter],
        owners: &[Address],
        verified: bool,
    ) -> anyhow::Result<Option<NftsPriceRangeRecord>> {
        let (attributes_filter, bind_params) = build_attributes_filter(2, attributes, "n")?;

        if verified && !owners.is_empty() {
            bail!("Filter by owners must use unverified instead verified");
        }

        if !verified {
            let query = format!(
                r#"
                select min(min_floor_price) as "from", max(min_floor_price) as "to"
                    from (
                             SELECT least(MIN(CASE WHEN ow.address IS NOT NULL THEN s.price * tup.usd_price END), MIN(
                                            CASE WHEN ow2.address IS NOT NULL THEN a.min_bid * tup2.usd_price END)) min_floor_price


                             FROM nft n
                                      LEFT JOIN nft_auction a ON a.nft = n.address AND a.status = 'active' AND
                                                                 (a.finished_at = to_timestamp(0) OR a.finished_at > NOW())
                                      LEFT JOIN offers_whitelist ow2 ON ow2.address = a.address
                                      LEFT JOIN token_usd_prices tup2 ON tup2.token = a.price_token
                                      LEFT JOIN nft_direct_sell s ON s.nft = n.address AND s.state = 'active' AND
                                                                     (s.expired_at = to_timestamp(0) OR s.expired_at > NOW())
                                      LEFT JOIN offers_whitelist ow ON ow.address = s.address
                                      LEFT JOIN token_usd_prices tup ON tup.token = s.price_token
                             WHERE NOT n.burned
                               and (n.owner = any($1::text[]) or $2 = '{{}}')
                               and (n.collection = any($2::text[]) or $2 = '{{}}')
                               {attributes_filter}
                             group by n.address
                         ) ag
            "#,
                attributes_filter = attributes_filter
            );

            let mut db_query = sqlx::query_as(&query);
            db_query = db_query.bind(owners);
            db_query = db_query.bind(collections);
            for (param1, param2) in bind_params {
                db_query = db_query.bind(param1).bind(param2);
            }

            db_query
                .fetch_optional(self.db.as_ref())
                .await
                .map_err(|e| anyhow!(e))
        } else {
            let query = format!(
                r#"
                SELECT
                    MIN(LEAST(n.floor_price_auc_usd, n.floor_price_sell_usd)) as "from",
                    MAX(LEAST(n.floor_price_auc_usd, n.floor_price_sell_usd)) as "to"
                FROM
                    nft_verified_extended n
                WHERE
                    (n.collection = ANY ($1) OR $1 = '{{}}')
                    {attributes_filter}
                HAVING
                    COALESCE(MIN(LEAST(n.floor_price_auc_usd, n.floor_price_sell_usd)), MAX(LEAST(n.floor_price_auc_usd, n.floor_price_sell_usd))) IS NOT NULL
            "#,
                attributes_filter = attributes_filter
            );

            let mut db_query = sqlx::query_as(&query);
            db_query = db_query.bind(collections);

            for (param1, param2) in bind_params {
                db_query = db_query.bind(param1).bind(param2);
            }

            db_query
                .fetch_optional(self.db.as_ref())
                .await
                .map_err(|e| anyhow!(e))
        }
    }
}

fn build_attributes_filter(
    mut index_from: usize,
    attributes: &[AttributeFilter],
    nft_table_alias: &str,
) -> Result<(String, Vec<(String, String)>), sqlx::Error> {
    let mut attributes_filter = String::default();
    let mut bind_params = Vec::new();

    for attribute in attributes.iter() {
        let index1 = index_from;
        let index2 = index_from + 1;
        index_from += 2;

        attributes_filter.push_str(&format!(
            r#" AND EXISTS(
            SELECT 1 FROM nft_attributes na
            WHERE
                na.nft = {nft_table_alias}.address AND (LOWER(na.trait_type) = LOWER(${index1}) AND LOWER(TRIM(na.value #>> '{{}}')::text ) = any(${index2}::text[]))
            )
        "#
        ));

        let values_as_text_array = format!(
            "{{{}}}",
            attribute
                .trait_values
                .iter()
                .map(|v| v.to_lowercase())
                .collect::<Vec<_>>()
                .join(",")
        );

        bind_params.push((attribute.trait_type.clone(), values_as_text_array));
    }

    Ok((attributes_filter, bind_params))
}
