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
       floor_price_usd.val                    as      floor_price_usd,
       last_deal.last_price                           deal_price_usd,
       coalesce(s.price, a.min_bid)           as      floor_price,
       coalesce(s.price_token, a.price_token) as      floor_price_token,
       n.id::text                             as      nft_id,
       case when $7 then count(1) over () else 0 end total_count
from nft n
         left join nft_metadata m on m.nft = n.address
         left join lateral ( select s.address
                             from nft_direct_buy s
                                      join offers_whitelist ow on ow.address = s.address
                                      join token_usd_prices tup on tup.token = s.price_token
                             where s.state = 'active'
                               and s.nft = n.address
                             group by s.address, (s.price * tup.usd_price)
                             having (s.price * tup.usd_price) = max(s.price * tup.usd_price)
                             limit 1 ) best_offer on true
         LEFT JOIN nft_auction a ON a.nft = n.address AND a.status = 'active' AND
                                    (a.finished_at = to_timestamp(0) OR a.finished_at > NOW())
         LEFT JOIN offers_whitelist ow2 ON ow2.address = a.address
         LEFT JOIN token_usd_prices tup2 ON tup2.token = a.price_token
         LEFT JOIN nft_direct_sell s ON s.nft = n.address AND s.state = 'active' AND
                                        (s.expired_at = to_timestamp(0) OR s.expired_at > NOW())
         LEFT JOIN offers_whitelist ow ON ow.address = s.address
         LEFT JOIN token_usd_prices tup ON tup.token = s.price_token

         LEFT JOIN LATERAL (
    SELECT LEAST(
                   CASE WHEN ow.address IS NOT NULL THEN s.price * tup.usd_price END,
                   CASE WHEN ow2.address IS NOT NULL THEN a.min_bid * tup2.usd_price END
               ) val ) floor_price_usd on true

         left join lateral ( select nph.price * tup.usd_price as last_price
                             from nft_price_history nph
                                      join offers_whitelist ow on ow.address = nph.source
                                      join token_usd_prices tup on tup.token = nph.price_token
                             where nph.nft = n.address
                             order by nph.ts desc
                             limit 1 ) last_deal on true
where (n.owner = any ($1) or $1 = '{}')
  and (n.collection = any ($2) or $2 = '{}')
  and (
        (not $3::bool and not $4::bool) or
        (ow2.address is not null and $3::bool) or
        (ow.address is not null and $4::bool)
    )
  #ATTRIBUTES#
  and (floor_price_usd.val between coalesce($8, floor_price_usd.val) and coalesce($9, floor_price_usd.val) or coalesce($8, $9) is null)
--   and ($8::varchar is null or n.address in (select nct.nft_address
--                                              from nft_type_mv nct
--                                              where nct.mimetype ilike $8
--                                                and ($9::boolean is false or nct.verified is true)))
#ORDER#
limit $5 offset $6