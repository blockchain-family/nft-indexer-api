select n.address,
       n.collection,
       n.owner,
       n.manager,
       n.name::text                           as      name,
       n.description,
       n.burned,
       n.updated,
       n.owner_update_lt                      as      tx_lt,
       m.meta,
       a.address                              as      auction,
       a.status::auction_status               as      auction_status,
       s.address                              as      forsale,
       s.state::direct_sell_state             as      forsale_status,
       best_offer.address                     as      best_offer,
       LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd)                    as      floor_price_usd,
       last_deal.last_price                           deal_price_usd,
       coalesce(s.price, a.min_bid)           as      floor_price,
       coalesce(s.price_token, a.price_token) as      floor_price_token,
       n.id::text                             as      nft_id,
       case when $7 then count(1) over () else 0 end total_count
from nft_verified_extended nve
     LEFT JOIN LATERAL (SELECT * FROM nft where nve.address = nft.address) n on true
         left join nft_metadata m on m.nft = n.address
         left join lateral (  select s.address
                                                                    from nft_direct_buy s
                                                                             left join token_usd_prices tup on tup.token = s.price_token
                                                                    where state = 'active'
                                                                      and nft = n.address
                                                                    order by s.price * tup.usd_price desc
                                                                    limit 1  ) best_offer on true
         LEFT JOIN nft_auction a ON a.nft = n.address AND a.status = 'active' AND
                                    (a.finished_at = to_timestamp(0) OR a.finished_at > NOW())
         LEFT JOIN offers_whitelist ow2 ON ow2.address = a.address
         LEFT JOIN token_usd_prices tup2 ON tup2.token = a.price_token
         LEFT JOIN nft_direct_sell s ON s.nft = n.address AND s.state = 'active' AND
                                        (s.expired_at = to_timestamp(0) OR s.expired_at > NOW())
         LEFT JOIN offers_whitelist ow ON ow.address = s.address
         LEFT JOIN token_usd_prices tup ON tup.token = s.price_token

         left join lateral ( select nph.price * tup.usd_price as last_price
                             from nft_price_history nph
                                      join offers_whitelist ow on ow.address = nph.source
                                      join token_usd_prices tup on tup.token = nph.price_token
                             where nph.nft = n.address
                             order by nph.ts desc
                             limit 1 ) last_deal on true
where (n.owner = any ($1) or $1 = '{}')
  and (nve.collection = any ($2) or $2 = '{}')
  and (
        (not $3::bool and not $4::bool) or
        (nve.floor_price_auc_usd is not null and $3::bool) or
        (nve.floor_price_sell_usd is not null and $4::bool)
    )

  and (coalesce($8, $9) is null or LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd) between coalesce(null, LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd)) and coalesce(null, LEAST(nve.floor_price_auc_usd, nve.floor_price_sell_usd)))
#ATTRIBUTES#
#ORDER#
limit $5 offset $6