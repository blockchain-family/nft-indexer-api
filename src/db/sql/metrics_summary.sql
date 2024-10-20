WITH periods AS (SELECT $1::timestamp AS date_from,
                        $2::timestamp AS date_to,
                        'current'     AS period_type
                 UNION ALL
                 SELECT $1::timestamp - ($2 - $1)::interval AS date_from,
                        $1::timestamp - INTERVAL '1 second' AS date_to,
                        'previous'                          AS period_type),
     total_volume AS (SELECT nph.collection                 AS address,
                             p.period_type,
                             SUM(nph.price * tup.usd_price) AS price_usd
                      FROM periods p
                               JOIN nft_price_history nph ON nph.ts BETWEEN p.date_from AND p.date_to
                               JOIN token_usd_prices tup ON tup.token = nph.price_token
                      GROUP BY nph.collection, p.period_type),
     aggregated_volumes AS (SELECT tv.address,
                                   SUM(CASE WHEN tv.period_type = 'current' THEN tv.price_usd ELSE 0 END)  AS total_volume_usd_now,
                                   SUM(CASE WHEN tv.period_type = 'previous' THEN tv.price_usd ELSE 0 END) AS total_volume_usd_previous
                            FROM total_volume tv
                            GROUP BY tv.address)
SELECT c.address                                  AS "collection!",
       c.name,
       c.logo,
       c.floor_price_usd                          AS "floor_price",
       COALESCE(av.total_volume_usd_now, 0)       AS "total_volume_usd_now!",
       COALESCE(av.total_volume_usd_previous, 0)  AS "total_volume_usd_previous!",
       c.owners_count::int                        AS "owners_count!",
       c.nft_count::int                           AS "nfts_count!",
       (SELECT COUNT(*) FROM nft_collection)::int AS "total_rows_count!"
FROM nft_collection_details c
         LEFT JOIN aggregated_volumes av ON av.address = c.address
WHERE c.verified
ORDER BY COALESCE(av.total_volume_usd_now, 0) DESC
LIMIT $3 OFFSET $4