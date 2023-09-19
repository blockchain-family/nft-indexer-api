with nfts as (
    select nvm.address,
           nvm.collection,
           nvm.owner,
           nvm.manager,
           nvm.name,
           nvm.description,
           nvm.burned,
           nvm.updated,
           nvm.owner_update_lt,
           nvm.id,
           null                    auction,
           null::auction_status    auction_status,
           null                    forsale,
           null::direct_sell_state forsale_status,
           null::numeric           floor_price_usd,
           null::numeric           floor_price,
           null                    floor_price_token

    from nft_verified_mv nvm
    where
      --only when sale type filters are disabled
        not $3
      and not $4
      and ((nvm.collection = any ($2) or $2 = '{}') and (nvm.owner = any ($1) or $1 = '{}'))
      and not burned
    order by nvm.name #ORDER_DIRECTION#, nvm.address
),

     deals as (
         select ag.address,
                ag.collection,
                ag.owner,
                ag.manager,
                ag.name,
                ag.description,
                ag.burned,
                ag.updated,
                ag.owner_update_lt,
                ag.id,
                ag.auction,
                ag.auction_status,
                ag.sell        as forsale,
                ag.sell_status as forsale_status,
                ag.price_usd      floor_price_usd,
                ag.price          floor_price,
                ag.token          floor_price_token

         from (
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
                         tup.token,
                         a.min_bid                 as price,
                         a.min_bid * tup.usd_price as price_usd,
                         a.address                 as auction,
                         a.status::auction_status  as auction_status,
                         null                         sell,
                         null::direct_sell_state   as sell_status

                  from nft_auction a
                           join nft_verified_mv n
                                on n.address = a.nft
                           join offers_whitelist ow on ow.address = a.address
                           left join token_usd_prices tup on tup.token = a.price_token
                  where ($3::bool or $8::bool)
                    and a.nft = n.address
                    and a.status = 'active'::auction_status
                    and (a.finished_at = to_timestamp(0) or a.finished_at > now()::timestamp)
                    and ($1 = '{}' or n.owner = any ($1::text[]))
                    and ($2 = '{}' or n.collection = any ($2))
                    and $9::bool


                  union all

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
                         tup.token,
                         s.price                 as price,
                         s.price * tup.usd_price as price_usd,
                         null                    as auction,
                         null                    as auction_status,
                         s.address                  sell,
                         s.state                 as sell_status

                  from nft_direct_sell s
                           join nft_verified_mv n
                                on n.address = s.nft
                           join offers_whitelist ow on ow.address = s.address
                           left join token_usd_prices tup on tup.token = s.price_token
                  where ($4::bool or $8::bool)
                    and s.state = 'active'::direct_sell_state
                    and (s.expired_at = to_timestamp(0) or s.expired_at > now())
                    and ($1 = '{}' or n.owner = any ($1::text[]))
                    and ($2 = '{}' or n.collection = any ($2))
                    and $9::bool
              ) ag

         order by #DEALS_ORDER_FIELD# #ORDER_DIRECTION#
     ),

     res as (
         select *
         from deals
         union all
         select *
         from nfts n
     )

select n.address,
       n.collection,
       n.owner,
       n.manager,
       n.name::text                                                        as name,
       n.description,
       n.burned,
       n.updated,
       n.owner_update_lt                                                   as tx_lt,
       m.meta,
       coalesce(n.auction, auc.auction)                                       auction,
       coalesce(n.auction_status, auc.status)::auction_status              as auction_status,
       coalesce(n.forsale, sale.forsale)                                   as forsale,
       coalesce(n.forsale_status, sale.status)::direct_sell_state          as forsale_status,
       best_offer.address                                                  as best_offer,
       coalesce(n.floor_price_usd, least(auc.price_usd, sale.price_usd))      floor_price_usd,
       last_deal.last_price                                                   deal_price_usd,
       coalesce(n.floor_price, case
                                   when least(auc.price_usd, sale.price_usd) = auc.price_usd then auc.min_bid
                                   when least(auc.price_usd, sale.price_usd) = sale.price_usd then sale.price
                                   else null::numeric end)                 as floor_price,
       coalesce(n.floor_price_token, case
                                         when least(auc.price_usd, sale.price_usd) = auc.price_usd
                                             then auc.token::character varying
                                         when least(auc.price_usd, sale.price_usd) = sale.price_usd
                                             then sale.token::character varying
                                         else null::character varying end) as floor_price_token,
       n.id::text                                                          as nft_id,
       case when $7 then count(1) over () else 0 end                         total_count
from res n
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

         left join lateral ( select a.address                 as auction,
                                    a.status                  as status,
                                    a.min_bid * tup.usd_price as price_usd,
                                    tup.token,
                                    a.min_bid
                             from nft_auction a
                                      join offers_whitelist ow on ow.address = a.address
                                      left join token_usd_prices tup on tup.token = a.price_token
                             where n.auction is null
                               and a.nft = n.address
                               and a.status = 'active'::auction_status
                               and (a.finished_at = to_timestamp(0) or a.finished_at > now()::timestamp)
                             limit 1 ) auc on true

         left join lateral ( select s.address               as forsale,
                                    s.state                 as status,
                                    s.price * tup.usd_price as price_usd,
                                    s.price,
                                    tup.token
                             from nft_direct_sell s
                                      join offers_whitelist ow on ow.address = s.address
                                      left join token_usd_prices tup on tup.token = s.price_token
                             where n.forsale is null
                               and s.nft = n.address
                               and s.state = 'active'::direct_sell_state
                               and (s.expired_at = to_timestamp(0) or s.expired_at > now())
                             limit 1 ) sale on true

         left join lateral ( select nph.price * tup.usd_price as last_price
                             from nft_price_history nph
                                      join offers_whitelist ow on ow.address = nph.source
                                      join token_usd_prices tup on tup.token = nph.price_token
                             where nph.nft = n.address
                             order by nph.ts desc
                             limit 1 ) last_deal on true

#ORDER_RESULT#
limit $5 offset $6
