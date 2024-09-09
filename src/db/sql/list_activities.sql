with events_whitelist as (select of.address
                          from deployed_offers of
                                   inner join roots r
                                              on r.address = of.root
                          union
                          select address
                          from roots)
select json_build_object(
               'totalRows',
               coalesce(max(r.total_rows), 0),
               'data',
               coalesce(json_agg(json_build_object(
                       'eventType', r.computed_event_kind,
                       'id', r.id,
                       'name', nft.name,
                       'description', nft.description,
                       'datetime', r.created_at,
                       'address', r.nft,
                       'previewUrl', nm.meta -> 'preview' ->> 'source',
                       'mint', case
                                   when r.computed_event_kind = 'mint' then
                                       json_build_object(
                                               'owner',
                                               r.args -> 'owner',
                                               'creator',
                                               r.args -> 'creator'
                                       )
                           end,
                       'transfer',
                       case
                           when r.computed_event_kind = 'transfer'
                               then json_build_object(
                                   'from', r.args -> 'old_owner',
                                   'to', r.args -> 'new_owner')
                           end,
                       'directSell',
                       case
                           when
                               r.event_type = 'direct_sell_state_changed'
                               then
                               json_build_object(
                                       'creator', r.args -> 'value2' -> 'creator',
                                       'startTime', r.args -> 'value2' -> 'start',
                                       'endTime', r.args -> 'value2' -> 'end',
                                       'status', r.args -> 'value2' -> 'status',
                                       'price', r.args -> 'value2' ->> '_price',
                                       'usdPrice', ((r.args -> 'value2' ->> '_price')::numeric * curr.usd_price)::text,
                                       'paymentToken', r.args -> 'value2' -> 'token',
                                       'newOwner', r.new_owner
                               )
                           end,
                       'directBuy',
                       case
                           when
                               r.event_type = 'direct_buy_state_changed'
                               then
                               json_build_object(
                                       'creator', r.args -> 'value2' -> 'creator',
                                       'startTime', r.args -> 'value2' -> 'start_time_buy',
                                       'endTime', r.args -> 'value2' -> 'end_time_buy',
                                       'durationTime', r.args -> 'value2' -> 'duration_time',
                                       'price', r.args -> 'value2' ->> '_price',
                                       'usdPrice', ((r.args -> 'value2' ->> '_price')::numeric * curr.usd_price)::text,
                                       'status', r.args -> 'value2' -> 'status',
                                       'spentToken', r.args -> 'value2' -> 'spent_token',
                                       'oldOwner', r.old_owner
                               )
                           end,
                       'auction',
                       case
                           when
                               computed_event_kind in
                               ('auction_active', 'auction_complete', 'auction_canceled', 'auction_bid_placed')
                               then
                               json_build_object(
                                       'auctionActive',
                                       case
                                           when
                                               r.computed_event_kind = 'auction_active'
                                               then
                                               json_build_object(
                                                       'nftOwner', r.args -> 'value0' -> 'subject_owner',
                                                       'auctionStartTime', r.args -> 'value0' -> 'start_time',
                                                       'auctionEndTime', r.args -> 'value0' -> 'end_time',
                                                       'auctionDuration', r.args -> 'value0' -> 'duration',
                                                       'state', 1,
                                                       'paymentToken', r.args -> 'value0' -> 'payment_token',
                                                       'price', r.args -> 'value0' ->> 'price',
                                                       'usdPrice',
                                                       ((r.args -> 'value0' ->> 'price')::numeric * curr.usd_price)::text
                                               )
                                           end,
                                       'auctionComplete',
                                       case
                                           when
                                               r.computed_event_kind = 'auction_complete'
                                               then
                                               json_build_object(
                                                       'nftOwner', r.args -> 'value2' -> 'subject_owner',
                                                       'auctionStartTime', r.args -> 'value2' -> 'start_time',
                                                       'auctionEndTime', r.args -> 'value2' -> 'end_time',
                                                       'auctionDuration', r.args -> 'value2' -> 'duration',
                                                       'state', 3,
                                                       'paymentToken', r.args -> 'value2' -> 'payment_token',
                                                       'maxBidValue', r.args ->> 'value',
                                                       'maxBidAddress', r.args -> 'buyer',
                                                       'price', (r.args ->> 'value'),
                                                       'usdPrice', ((r.args ->> 'value')::numeric * curr.usd_price)::text
                                               )
                                           end,
                                       'auctionCanceled',
                                       case
                                           when
                                               r.computed_event_kind = 'auction_canceled'
                                               then
                                               json_build_object(
                                                       'nftOwner', r.args -> 'value0' -> 'subject_owner',
                                                       'auctionStartTime', r.args -> 'value0' -> 'start_time',
                                                       'auctionEndTime', r.args -> 'value0' -> 'end_time',
                                                       'auctionDuration', r.args -> 'value0' -> 'duration',
                                                       'state', 0,
                                                       'paymentToken', r.args -> 'value0' -> 'payment_token',
                                                       'price', r.args -> 'value0' ->> 'price',
                                                       'usdPrice',
                                                       ((r.args -> 'value0' ->> 'price')::numeric * curr.usd_price)::text
                                               )
                                           end,
                                       'auctionBidPlaced',
                                       case
                                           when
                                               r.computed_event_kind = 'auction_bid_placed'
                                               then
                                               json_build_object(
                                                       'bidSender', r.args -> 'buyer',
                                                       'paymentToken', r.args -> 'value3' -> 'payment_token',
                                                       'bidValue', r.args ->> 'value',
                                                       'usdPrice', ((r.args ->> 'value')::numeric * curr.usd_price)::text
                                               )
                                           end
                               )
                           end
                                 )), '[]'::json)
       ) content
from get_events(
             p_owner => $2::t_address,
             p_event_kind => $1::event_kind[],
             p_nft => $3::t_address,
             p_collections => $4::t_address[],
             p_limit=> $5::integer,
             p_offset => $6::integer,
             p_with_count => $7::boolean,
             p_verified => $8::boolean) as r
         join nft on nft.address = r.nft
         left join nft_metadata nm on nm.nft = r.nft
         left join lateral (
    select p.usd_price
    from token_usd_prices p
    where r.args -> 'value2' ->> 'token' = p.token::text
       or r.args -> 'value2' ->> 'spent_token' = p.token::text
       or r.args -> 'value0' ->> '_payment_token' = p.token::text
       or r.args -> 'value2' ->> 'payment_token' = p.token::text
       or r.args -> 'value0' ->> 'payment_token' = p.token::text
       or r.args -> 'value3' ->> 'payment_token' = p.token::text
       or r.args -> 'value1' ->> 'payment_token' = p.token::text
    ) curr on true