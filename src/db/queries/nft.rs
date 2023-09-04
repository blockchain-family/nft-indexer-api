use crate::db::queries::Queries;
use crate::db::NftDetails;
use chrono::NaiveDateTime;

use super::*;

use crate::model::OrderDirection;
use sqlx::{self};
use std::fmt::Write;
use crate::handlers::nft::{AttributeFilter, NFTListOrder, NFTListOrderField};

impl Queries {
    pub async fn search_all(&self, search_str: &String) -> sqlx::Result<Vec<SearchResult>> {
        sqlx::query_as!(
            SearchResult,
            r#"
                select ag.address as "address!", nft_name, collection_name, object_type as "object_type!", image
                from (
                     select n.address,
                            n.name     nft_name,
                            nc.name    collection_name,
                            'nft'   as object_type,
                            CASE
                                WHEN m.meta is not null THEN m.meta::jsonb -> 'preview' ->> 'source'
                                END as "image",
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
                                else 1
                                end    priority
                     from nft n
                              left join nft_metadata m on n.address = m.nft
                              join nft_collection nc on n.collection = nc.address and nc.verified
                     where (n.name ilike '%' || $1 || '%'
                         or n.description ilike '%' || $1 || '%'
                         or n.address ilike '%' || $1 || '%')
                       and not n.burned

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
                                else 2
                                end         priority
                     from nft_collection c
                     where (c.name ilike '%' || $1 || '%'
                         or c.description ilike '%' || $1 || '%'
                         or c.address ilike '%' || $1 || '%')
                       and c.verified
                     ) ag
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
                SELECT n.*, 1::bigint as "total_count!"
                FROM nft_details n
                WHERE n.address = $1
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
                SELECT n.*, 1::bigint as "total_count!"
                FROM nft_details n
                WHERE n.address = ANY($1)
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
            from nft_details n
            join nft_collection nc
                 on nc.address = n.collection and
                    nc.verified
            left join lateral (
                select count(1) as cnt
                from nft_auction na
                join roots r on na.root = r.address
                where n.address = na.nft and
                      na.status = 'completed' and
                      na.finished_at >= $1
            ) auc on true
            left join lateral (
                select count(1) as cnt
                from nft_direct_sell na
                join roots r on na.root = r.address
                where n.address = na.nft and
                      na.state = 'filled' and
                      na.updated >= $1
            ) ds on true
            left join lateral (
                select count(1) as cnt
                from nft_direct_buy na
                join roots r on na.root = r.address
                where n.address = na.nft and
                      na.state = 'filled' and
                      na.updated >= $1
            ) db on true
            where n.updated >= $1 and
                  auc.cnt + ds.cnt + db.cnt > 0
            order by auc.cnt + ds.cnt + db.cnt desc, n.updated desc, n.address desc
            limit $2
            offset $3
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
        _price_from: Option<u64>,
        _price_to: Option<u64>,
        _price_token: Option<Address>,
        forsale: Option<bool>,
        auction: Option<bool>,
        verified: Option<bool>,
        limit: usize,
        offset: usize,
        attributes: &Vec<AttributeFilter>,
        order: Option<NFTListOrder>,
        with_count: bool,
    ) -> sqlx::Result<Vec<NftDetails>> {
        let mut sql = r#"
            select
                n.*,
                n."auction_status: _" as auction_status,
                n."forsale_status: _" as forsale_status,
                case
                    when $8 then
                        count(1) over ()
                    else
                        0
                end total_count
            from nft_details n
            inner join nft_collection c
                on n.collection = c.address
            where
                (n.owner = any($1) or array_length($1::varchar[], 1) is null) and
                (n.collection = ANY($2) or array_length($2::varchar[], 1) is null) and
                n.burned = false and
                (
                    ($3::bool is null and $4::bool is null) or
                    (
                        $3::bool is not null and
                        $4::bool is not null and
                        (
                            (
                                $4::bool and
                                n.forsale is not null and
                                n."forsale_status: _" = 'active' and
                                exists (
                                    select 1
                                    from nft_direct_sell nds
                                    where nds.nft = n.address and
                                          nds.created <= now() and
                                          nds.state = 'active' and
                                          (nds.expired_at = to_timestamp(0) or
                                           nds.expired_at > now())
                                )
                            ) or
                            (
                                not $4::bool and
                                n.forsale is null
                            ) or
                            (
                                $3::bool and
                                n.auction is not null and
                                n."auction_status: _" = 'active'
                            ) or
                            (
                                not $3::bool and
                                n.auction is null
                            )
                        )
                    ) or
                    (
                        $3::bool is null and
                        (
                            (
                                $4::bool and n.forsale is not null and
                                n."forsale_status: _" = 'active' and
                                exists (
                                    select 1
                                    from nft_direct_sell nds
                                    where nds.nft = n.address and
                                          nds.created <= now() and
                                          nds.state = 'active' and
                                          (nds.expired_at = to_timestamp(0) or
                                           nds.expired_at > now())
                                )
                            ) or
                            (
                                not $4::bool and
                                n.forsale is null
                            )
                        )
                    ) or
                    (
                        $4::bool is null and
                        (
                            (
                                $3::bool and
                                n.auction is not null and
                                n."auction_status: _" = 'active'
                            ) or
                            (
                                not $3::bool and
                                n.auction is null
                            )
                        )
                    )
                ) and
                (
                    $5::boolean is false or
                    c.verified is true
                )
        "#
            .to_string();

        for attribute in attributes {
            let values = attribute
                .trait_values
                .iter()
                .enumerate()
                .map(|it| format!("'{}'", it.1.to_lowercase()))
                .collect::<Vec<String>>()
                .join(",");
            // TODO need index trim(na.value #>> '{}')
            let _ = write!(
                sql,
                r#" and exists(
                    select 1 from nft_attributes na
                    where
                        na.nft = n.address and (
                            lower(na.trait_type) = lower('{0}') and
                            lower(trim(na.value #>> '{{}}')) in ({1})
                        )
                )
            "#,
                attribute.trait_type, values
            );
        }

        match order {
            None => {
                let _ = write!(
                    sql,
                    r#"
                        order by n.name, n.address
                    "#
                );
            }
            Some(order) => {
                let field = order.field.to_string();

                match order.direction {
                    OrderDirection::Asc => {
                        let _ = write!(sql, "order by n.{field}, n.name");
                    }
                    OrderDirection::Desc => match order.field {
                        NFTListOrderField::Name => {
                            let _ = write!(sql, "order by n.name desc");
                        }
                        _ => {
                            let _ =
                                write!(sql, "order by coalesce(n.{field}, 0) desc, n.name desc");
                        }
                    },
                }
            }
        };

        let _ = write!(
            sql,
            r#"
                limit $6
                offset $7
            "#
        );

        sqlx::query_as(&sql)
            .bind(owners)
            .bind(collections)
            .bind(auction)
            .bind(forsale)
            .bind(verified)
            .bind(limit as i64)
            .bind(offset as i64)
            .bind(with_count)
            .fetch_all(self.db.as_ref())
            .await
    }


    pub async fn get_traits(&self, nft: &Address) -> sqlx::Result<Vec<NftTraitRecord>> {
        sqlx::query_as!(
            NftTraitRecord,
            r#"
                WITH nft_attributes AS (
                    SELECT jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' AS trait_type,
                           jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      AS trait_value,
                           nm.meta,
                           n.collection                                                  AS nft_collection,
                           nm.nft
                    FROM nft_metadata nm
                             JOIN nft n ON n.address = nm.nft
                    WHERE nm.meta -> 'attributes' IS NOT NULL
                      AND nm.nft = $1
                ),
                     nft_attributes_col AS (
                         SELECT jsonb_array_elements(nm.meta -> 'attributes') -> 'trait_type' AS trait_type,
                                jsonb_array_elements(nm.meta -> 'attributes') -> 'value'      AS trait_value,
                                nm.nft
                         FROM nft_metadata nm
                         where nm.nft in (
                             select n2.address
                             from nft n2
                                      join nft n3 on n3.address = $1
                                 and n2.collection = n3.collection
                         )
                     )
                SELECT (na.trait_type #>> '{}')::text as trait_type, (na.trait_value #>> '{}')::text as trait_value, COUNT(*) as "cnt!"
                FROM nft_attributes na
                         LEFT JOIN nft_attributes_col na2
                                   ON na.trait_type = na2.trait_type
                                       AND na.trait_value = na2.trait_value
                GROUP BY na.trait_type, na.trait_value
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
                 select
                    ag.dt as "ts!",
                    (price * tup.usd_price) as "usd_price!"
                 from (
                    select
                        t.finished_at dt,
                        t.price_token,
                        t.price
                    from nft_direct_sell t
                    join roots r
                        on t.root = r.address
                    where t.nft = $1 and
                          t.finished_at between $2 and $3 and
                          t.state = 'filled'
                    union all
                    select
                        t.finished_at,
                        t.price_token,
                        t.price
                    from nft_direct_buy t
                    join roots r
                        on t.root = r.address
                    where t.nft = $1 and
                          t.finished_at between $2 and $3 and
                          t.state = 'filled'
                    union all
                    select
                        t.finished_at,
                        t.price_token,
                        t.max_bid
                    from nft_auction t
                    join roots r
                        on t.root = r.address
                    where t.nft = $1 and
                          t.finished_at between $2 and $3 and
                          t.status = 'completed'
                 ) as ag
                 join token_usd_prices tup
                      on tup.token = ag.price_token
                 where (price * tup.usd_price) is not null
                 order by 1
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
                select nd.*,
                       nd."auction_status: _" as auction_status,
                       nd."forsale_status: _" as forsale_status,
                       0::int8                   total_count,
                       ag.collection
                from (
                    select n.*
                    from nft n
                    join nft_collection c
                        on n.collection = c.address
                    join nft_direct_sell nds
                         on nds.nft = n.address and
                            nds.created <= now() and
                            (now() <= nds.expired_at or nds.expired_at = to_timestamp(0)) and
                            nds.state = 'active'
                            and nds.price <= $1
                    inner join roots r
                        on nds.root = r.address
                    where n.burned is false and
                          c.verified is true
                    order by random()
                    limit $2
                ) ag
                join nft_details nd
                    on nd.address = ag.address

            "#,
        )
            .bind(max_price)
            .bind(limit)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn nft_sell_count(&self, max_price: i64) -> sqlx::Result<Option<i64>> {
        sqlx::query_scalar!(
            r#"
            select
                count(1)
            from nft n
            join nft_collection c
                on n.collection = c.address
            join nft_direct_sell nds
                on nds.nft = n.address and
                   nds.created <= now() and
                   (now() <= nds.expired_at or nds.expired_at = to_timestamp(0)) and
                    nds.state = 'active' and
                    nds.price <= $1::int8
            join roots r
                on nds.root = r.address
            where n.burned = false and
                  c.verified = true
           "#,
            max_price
        )
            .fetch_one(self.db.as_ref())
            .await
    }

    pub async fn nft_attributes_dictionary(&self) -> sqlx::Result<Vec<TraitDef>> {
        sqlx::query_as!(
            TraitDef,
            "
        select a.collection, a.trait_type, jsonb_agg(a.value) as values
        from (
            select distinct a.collection, a.trait_type, a.value
            from nft_attributes a
            where a.collection is not null and a.trait_type is not null
            order by a.collection ASC, a.trait_type ASC, a.value ASC
        ) as a
        group by a.collection, a.trait_type
        "
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
            "
        select distinct a.nft
        from nft_attributes a
        where a.collection = $1
        and a.trait_type = $2
        and a.value = ANY($3::jsonb[])
        order by 1 asc",
            collection,
            trait_type,
            values
        )
            .fetch_all(self.db.as_ref())
            .await
            .map(|x| x.iter().map(|y| y.nft.clone()).collect())
    }



}
