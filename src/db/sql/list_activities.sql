/* event cat, event type, owner, nft, collection, offset, limit */


with result as (
    select ne.*,
           (ne.args ->> 'from')::int             f,
           (ne.args ->> 'to')::int               t,
           nm.meta -> 'preview' ->> 'source'  as preview_url,
           n.description,
           n.name,
           n.owner,
           nm.meta,
           auction.args                          auction_args,
           case when $8 then
           count(1) over () else 0 end total_rows,
           direct_sell_chaned_owner.new_owner as new_owner,
           direct_buy_chaned_owner.old_owner  as old_owner
    from nft_events ne
             join nft n on ne.nft = n.address
             join nft_metadata nm on ne.nft = nm.nft
             left join lateral (
        select n.args
        from nft_events n
        join events_whitelist ew on n.address = ew.address
        where n.event_cat = ne.event_cat
          and n.event_type = 'auction_active'
          and n.address = ne.address
          and n.created_lt < ne.created_lt
          and ne.event_type in ('auction_complete', 'auction_cancelled', 'auction_bid_placed')
        order by n.created_lt
        limit 1
        ) auction on true

             left join lateral (
        select n.args ->> 'new_owner' new_owner
        from nft_events n
            join events_whitelist ew on n.address = ew.address
        where n.event_cat = 'nft'
          and n.event_type = 'nft_owner_changed'
          and n.nft = ne.nft
          and n.created_lt >= ne.created_lt
          and ((ne.args ->> 'from')::int = 2 and (ne.args ->> 'to')::int = 3 and
               ('SellPurchased' = any ($2) and ne.event_cat = 'direct_sell'))
          and n.args ->> 'old_owner' = ne.args ->> 'creator'
        order by n.created_lt
        limit 1
        ) direct_sell_chaned_owner on true
             left join lateral (
        select n.args ->> 'old_owner' old_owner
        from nft_events n
                join events_whitelist ew on n.address = ew.address
        where n.event_cat = 'nft'
          and n.event_type = 'nft_owner_changed'
          and n.nft = ne.nft
          and n.created_lt >= ne.created_lt
          and ((ne.args ->> 'from')::int = 2 and (ne.args ->> 'to')::int = 3 and
               ('OfferFilled' = any ($2) and ne.event_cat = 'direct_buy'))
          and n.args ->> 'new_owner' = ne.args ->> 'creator'
        order by n.created_lt
        limit 1
        ) direct_buy_chaned_owner on true
    where ((
                $3 in (ne.args ->> 'subject_owner',
                        ne.args ->> 'creator',
                        ne.args ->> 'buyer',
                        ne.args ->> 'seller',
                        ne.args ->> 'old_owner',
                        ne.args ->> 'new_owner',
                        auction.args ->> 'subject_owner',
                        direct_buy_chaned_owner.old_owner,
                        direct_sell_chaned_owner.new_owner
                )
            or $3 is null))
      and (exists(select 1 from  events_whitelist ew where ne.address = ew.address ) or (ne.event_type in ('nft_owner_changed', 'nft_created')))
      and (ne.nft = $4 or $4 is null)
      and (n.collection = any ($5) or $5 = '{}')
      and (ne.event_cat::text = any ($1) or $1 = '{}')
      and (
            ((ne.args ->> 'from')::integer = 0 and (ne.args ->> 'to')::integer = 2) or
            ((ne.args ->> 'from')::integer = 2 and (ne.args ->> 'to')::integer = 3) or
            ((ne.args ->> 'from')::integer = 2 and (ne.args ->> 'to')::integer = 4)
            or
            ne.event_type in ('auction_active',
                              'auction_cancelled',
                              'auction_bid_placed',
                              'auction_complete',
                              'nft_created',
                              'nft_owner_changed')
        )
      and (
            ('Mint' = any ($2) and ne.event_type = 'nft_created') or
            ('Transfer' = any ($2) and ne.event_type = 'nft_owner_changed') or
            ('AuctionActive' = any ($2) and ne.event_type = 'auction_active') or
            ('AuctionBidPlaced' = any ($2) and ne.event_type = 'auction_bid_placed') or
            ('AuctionCanceled' = any ($2) and ne.event_type = 'auction_cancelled') or
            ('AuctionComplete' = any ($2) and ne.event_type = 'auction_complete')
            or
            ((ne.args ->> 'from')::int = 0 and (ne.args ->> 'to')::int = 2 and
             (('OfferActive' = any ($2) and ne.event_cat = 'direct_buy') or
              ('SellActive' = any ($2) and ne.event_cat = 'direct_sell')))
            or
            ((ne.args ->> 'from')::int = 2 and (ne.args ->> 'to')::int = 3 and (
                    ('OfferFilled' = any ($2) and ne.event_cat = 'direct_buy') or
                    ('SellPurchased' = any ($2) and ne.event_cat = 'direct_sell')))
            or
            ((ne.args ->> 'from')::int = 2 and (ne.args ->> 'to')::int = 4 and (
                    ('SellCanceled' = any ($2) and (ne.event_cat = 'direct_sell')) or
                    ('OfferCanceled' = any ($2) and ne.event_cat = 'direct_buy'))
                )
            or ($2) = '{}'
        )
    order by ne.created_at desc, ne.id desc
    limit $6 offset $7
)
select json_build_object('totalRows', coalesce(max(r.total_rows), 0), 'data',
                         coalesce(json_agg(json_build_object('eventType', case
                                                                              when r.f = 0 and r.t = 2 and r.event_cat = 'direct_sell'
                                                                                  then 'sell_active'
                                                                              when r.f = 0 and r.t = 2 and r.event_cat = 'direct_buy'
                                                                                  then 'offer_active'
                                                                              when r.f = 2 and r.t = 3 and r.event_cat = 'direct_sell'
                                                                                  then 'sell_purchased'
                                                                              when r.f = 2 and r.t = 3 and r.event_cat = 'direct_buy'
                                                                                  then 'offer_filled'
                                                                              when r.f = 2 and r.t = 4 and r.event_cat = 'direct_sell'
                                                                                  then 'sell_canceled'
                                                                              when r.f = 2 and r.t = 4 and r.event_cat = 'direct_buy'
                                                                                  then 'offer_canceled'
                                                                              when r.event_type = 'nft_created'
                                                                                  then 'mint'
                                                                              when r.event_type = 'nft_owner_changed'
                                                                                  then 'transfer'
                                                                              when r.event_type = 'auction_cancelled'
                                                                                  then 'auction_canceled'
                                                                              else r.event_type::text
                             end,
                                                             'id', r.id,
                                                             'name', r.name,
                                                             'description', r.description,
                                                             'datetime', r.created_at,
                                                             'address', r.nft,
                                                             'previewUrl', r.preview_url,
                                                             'mint',
                                                             case
                                                                 when r.event_type = 'nft_created'
                                                                     then json_build_object('owner', r.args -> 'owner',
                                                                                            'creator',
                                                                                            r.args -> 'creator')
                                                                 end,
                                                             'transfer',
                                                             case
                                                                 when r.event_type = 'nft_owner_changed'
                                                                     then json_build_object('from',
                                                                                            r.args -> 'old_owner',
                                                                                            'to', r.args -> 'new_owner')
                                                                 end,
                                                             'directSell',
                                                             case
                                                                 when r.event_cat = 'direct_sell' then
                                                                     json_build_object(
                                                                             'creator', r.args -> 'creator',
                                                                             'startTime', r.args -> 'start',
                                                                             'endTime', r.args -> 'end',
                                                                             'status', r.args -> 'status',
                                                                             'price', r.args -> '_price',
                                                                             'usdPrice',
                                                                             ((r.args ->> '_price')::numeric * curr.usd_price)::text,
                                                                             'paymentToken', r.args -> 'token',
                                                                             'newOwner', r.new_owner
                                                                         )
                                                                 end,
                                                             'directBuy',
                                                             case
                                                                 when r.event_cat = 'direct_buy' then
                                                                     json_build_object(
                                                                             'creator', r.args -> 'creator',
                                                                             'startTime', r.args -> 'start_time_buy',
                                                                             'endTime', r.args -> 'end_time_buy',
                                                                             'durationTime', r.args -> 'duration_time',
                                                                             'price', r.args -> '_price',
                                                                             'usdPrice',
                                                                             ((r.args ->> '_price')::numeric * curr.usd_price)::text,
                                                                             'status', r.args -> 'status',
                                                                             'spentToken', r.args -> 'spent_token',
                                                                             'oldOwner', r.old_owner
                                                                         )
                                                                 end,
                                                             'auction',
                                                             case
                                                                 when event_cat = 'auction' then
                                                                     json_build_object(
                                                                             'auctionActive',
                                                                             case
                                                                                 when r.event_type = 'auction_active'
                                                                                     then
                                                                                     json_build_object(
                                                                                             'nftOwner',
                                                                                             r.args -> 'subject_owner'
                                                                                         , 'auctionStartTime',
                                                                                             r.args -> 'start_time'
                                                                                         , 'auctionEndTime',
                                                                                             r.args -> 'finish_time'
                                                                                         , 'auctionDuration',
                                                                                             r.args -> 'duration'
                                                                                         , 'state', r.args -> 'status'
                                                                                         , 'paymentToken',
                                                                                             r.args -> '_payment_token'
                                                                                         , 'price', r.args -> '_price'
                                                                                         , 'usdPrice',
                                                                                             ((r.args ->> '_price')::numeric * curr.usd_price)::text
                                                                                         )
                                                                                 end,
                                                                             'auctionComplete',
                                                                             case
                                                                                 when r.event_type = 'auction_complete'
                                                                                     then
                                                                                     json_build_object('nftOwner',
                                                                                                       r.args -> 'seller'
                                                                                         , 'auctionStartTime',
                                                                                                       r.auction_args -> 'start_time'
                                                                                         , 'auctionEndTime',
                                                                                                       r.auction_args -> 'finish_time'
                                                                                         , 'auctionDuration',
                                                                                                       r.auction_args -> 'duration'
                                                                                         , 'state',
                                                                                                       r.auction_args -> 'status'
                                                                                         , 'paymentToken',
                                                                                                       r.auction_args -> '_payment_token'
                                                                                         , 'maxBidValue',
                                                                                                       r.args -> 'value'
                                                                                         , 'maxBidAddress',
                                                                                                       r.args -> 'buyer'
                                                                                         , 'price', (r.args ->> 'value')
                                                                                         , 'usdPrice',
                                                                                                       ((r.args ->> 'value')::numeric * curr.usd_price)::text
                                                                                         )
                                                                                 end,
                                                                             'auctionCanceled',
                                                                             case
                                                                                 when r.event_type = 'auction_cancelled'
                                                                                     then
                                                                                     json_build_object(
                                                                                             'nftOwner',
                                                                                             r.auction_args -> 'subject_owner'
                                                                                         , 'auctionStartTime',
                                                                                             r.auction_args -> 'start_time'
                                                                                         , 'auctionEndTime',
                                                                                             r.auction_args -> 'finish_time'
                                                                                         , 'auctionDuration',
                                                                                             r.auction_args -> 'duration'
                                                                                         , 'state',
                                                                                             r.auction_args -> 'status'
                                                                                         , 'paymentToken',
                                                                                             r.auction_args -> '_payment_token'
                                                                                         , 'price',
                                                                                             r.auction_args -> '_price'
                                                                                         , 'usdPrice',
                                                                                             ((r.auction_args ->> '_price')::numeric * curr.usd_price)::text
                                                                                         )
                                                                                 end,
                                                                             'auctionBidPlaced',
                                                                             case
                                                                                 when r.event_type = 'auction_bid_placed'
                                                                                     then
                                                                                     json_build_object(
                                                                                             'bidSender',
                                                                                             r.args -> 'buyer'
                                                                                         , 'paymentToken',
                                                                                             r.auction_args -> '_payment_token'
                                                                                         , 'bidValue', r.args -> 'value'
                                                                                         , 'usdPrice',
                                                                                             ((r.args ->> 'value')::numeric * curr.usd_price)::text
                                                                                         )
                                                                                 end
                                                                         ) end
                             )
                                      ), '[]'::json))
           content
from result as r
         left join lateral (
    select p.usd_price
    from token_usd_prices p
    where r.args ->> 'token' = p.token::text
       or r.args ->> 'spent_token' = p.token::text
       or r.auction_args ->> '_payment_token' = p.token::text
       or r.args ->> '_payment_token' = p.token::text
    ) curr on true
