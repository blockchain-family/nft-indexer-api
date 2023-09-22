with periods as ( select $1::timestamp as date_from, $2::timestamp as date_to, 'current' as period_type
                  union all
                  select $1::timestamp - ($2::timestamp - $1::timestamp)::interval as date_from,
                         $1::timestamp - interval '1 seconds'                      as date_to,
                         'previous'                                                as period_type )
select c.address                                       as "collection!",
       c.name,
       c.logo,
       c.floor_price_usd                               as "floor_price",
       coalesce(total_volume.cur, 0)                   as "total_volume_usd_now!",
       coalesce(total_volume.prev, 0)                  as "total_volume_usd_previous!",
       c.owners_count::int                             as "owners_count!",
       c.nft_count::int                                as "nfts_count!",
       (count(1) over ())::int                         as "total_rows_count!"
from nft_collection_details c
         left join lateral ( select sum(case when ag.period_type = 'current' then ag.price_usd else 0 end)  cur,
                                    sum(case when ag.period_type = 'previous' then ag.price_usd else 0 end) prev
                             from (select p.period_type,
                                          case when n.address is not null then tup.usd_price * ndb.price else 0 end as price_usd
                                   from periods p
                                            left join nft_direct_buy ndb
                                                      on ndb.updated between p.date_from and p.date_to and ndb.state = 'filled'
                                            left join offers_whitelist r on r.address = ndb.address
                                            left join token_usd_prices tup on tup.token = ndb.price_token
                                            left join nft n on ndb.nft = n.address and n.collection = c.address and
                                                               not n.burned and r.address is not null
                                   union all
                                   select p.period_type,
                                          case when n.address is not null then tup.usd_price * nds.price else 0 end as price_usd
                                   from periods p
                                            left join nft_direct_sell nds
                                                      on nds.state = 'filled' and nds.updated between p.date_from and p.date_to
                                            left join token_usd_prices tup on tup.token = nds.price_token
                                            left join offers_whitelist r on r.address = nds.address
                                            left join nft n on nds.nft = n.address and n.collection = c.address and
                                                               not n.burned and r.address is not null
                                   union all
                                   select p.period_type,
                                          case
                                              when n.address is not null then tup.usd_price * na.max_bid
                                              else 0 end as price_usd
                                   from periods p
                                            left join public.nft_auction na on na.status = 'completed' and
                                                                               na.finished_at between p.date_from and p.date_to
                                            left join offers_whitelist r on r.address = na.address
                                            left join nft n on na.nft = n.address and not n.burned and
                                                               r.address is not null and n.collection = c.address
                                            left join token_usd_prices tup on tup.token = na.price_token) as ag ) as total_volume
                   on true
where c.verified
order by coalesce(total_volume.cur, 0) desc
limit $3 offset $4