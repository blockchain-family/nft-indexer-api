use crate::db::queries::Queries;

use super::*;

use crate::handlers::collection::CollectionListOrder;
use sqlx::{self};

impl Queries {
    pub async fn get_collection(
        &self,
        address: &String,
    ) -> sqlx::Result<Option<NftCollectionDetails>> {
        sqlx::query_as!(
            NftCollectionDetails,
            r#"
            select c.*, null::numeric as max_price, null::numeric as total_price, 1::bigint as "cnt!", '[]'::json as "previews!"
            from nft_collection_details c
            where c.address = $1
            "#,
            address
        )
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn collect_collections(&self, ids: &[String]) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(
            NftCollection,
            r#"
            select coalesce(ncc.address, c.address)                                                                          as "address!",
                   c.owner                                                                                                   as "owner!",
                   coalesce(ncc.name, c.name)                                                                                as "name",
                   coalesce(ncc.description, c.description)                                                                  as "description",
                   coalesce(ncc.updated, c.updated)                                                                          as "updated!",
                   coalesce(ncc.wallpaper, c.wallpaper)                                                                      as "wallpaper",
                   coalesce(ncc.logo, c.logo)                                                                                as "logo",
                   null::numeric                                                                                             as total_price,
                   null::numeric                                                                                             as max_price,
                   ( select count(*)
                     from ( select distinct owner
                            from nft n
                            where n.collection = c.address
                              and not n.burned ) owners )::int                                                               as owners_count,
                   c.verified                                                                                                as "verified!",
                   c.created                                                                                                 as "created!",
                   c.first_mint,
                   ( select count(*)
                     from nft n
                     where n.collection = c.address
                       and not n.burned )                                                                                    as "nft_count!",
                   count(1) over ()                                                                                          as "cnt!"
            from nft_collection c
                     left join nft_collection_custom ncc on c.address = ncc.address
            where c.address = any ($1)
              and owner is not null
            "#,
            ids
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_collections_by_owner(
        &self,
        owner: &String,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollection>> {
        sqlx::query_as!(
            NftCollection,
            r#"
            select c.address                                                                                            as "address!",
                   c.owner                                                                                              as "owner!",
                   c.name,
                   c.description,
                   c.updated                                                                                            as "updated!",
                   c.wallpaper,
                   c.logo,
                   null::numeric                                                                                        as total_price,
                   null::numeric                                                                                        as max_price,
                   ( select count(*)
                     from ( select distinct owner
                            from nft n
                            where n.collection = c.address
                              and not n.burned ) owners )::int                                                          as owners_count,
                   c.verified                                                                                           as "verified!",
                   c.created                                                                                            as "created!",
                   c.first_mint,
                   ( select count(*)
                     from nft n
                     where n.collection = c.address
                       and not n.burned )                                                                               as "nft_count!",
                   count(1) over ()                                                                                     as "cnt!"
            from nft_collection c
            where c.owner = $1
            limit $2 offset $3
            "#,
            owner,
            limit as i64,
            offset as i64
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn list_collections(
        &self,
        name: Option<&String>,
        owners: &[String],
        verified: Option<&bool>,
        collections: &[Address],
        limit: usize,
        offset: usize,
        order: Option<CollectionListOrder>,
    ) -> sqlx::Result<Vec<NftCollectionDetails>> {
        let order = match order {
            None => "c.owners_count desc".to_string(),
            Some(order) => {
                let field = order.field.to_string();
                format!("c.{field} {}", order.direction)
            }
        };

        let query = format!(
            r#"
            with collection_info as ( select c.address,
                                             c.owner,
                                             c.name,
                                             c.description,
                                             c.created,
                                             c.updated,
                                             c.wallpaper,
                                             c.logo,
                                             ( select count(*)
                                               from ( select distinct owner
                                                      from nft n
                                                      where n.collection = c.address and not n.burned ) owners )            as owners_count,
                                             c.verified,
                                             ( select count(*)
                                               from nft n
                                               where n.collection = c.address
                                                 and not n.burned )                                                         as nft_count,
                                             c.first_mint,
                                             count(1) over ()                                                               as "cnt",
                                             null::numeric                                                                  as max_price,
                                             null::numeric                                                                  as total_price
                                      from nft_collection c
                                      where (c.owner = any ($3) or array_length($3::varchar[], 1) is null)
                                        and ($4::boolean is false or c.verified is true)
                                        and ($5::varchar is null or c.name ilike $5)
                                        and (c.address = any ($6) or array_length($6::varchar[], 1) is null) )
            select c.*,
                   least(ds.price * tup_ds.usd_price, na.min_bid * tup_na.usd_price) as floor_price_usd,
                   ( select sum(coalesce(tup.usd_price * nph.price, 0)) as usd
                     from nft_price_history nph
                              join offers_whitelist ow on ow.address = nph.source
                              left join token_usd_prices tup on tup.token = nph.price_token
                     where nph.collection = c.address )                              as total_volume_usd,
                   ( select json_agg(res.json)
                     from ( select json_build_object('traitType', na.trait_type, 'traitValues',
                                                     json_agg(distinct trim(both from (na.value #>> '{{}}'::text[])))) as json
                            from nft_attributes na
                            where na.collection = c.address
                            group by na.trait_type, na.collection ) res )            as attributes,
                   ( select json_agg(ag2.preview_url) as previews
                     from ( select ag.preview_url
                            from ( select nm.meta -> 'preview' as preview_url
                                   from nft n
                                            join nft_metadata nm on n.address = nm.nft
                                   where n.collection = c.address
                                     and not n.burned
                                   limit 50 ) ag
                            order by random()
                            limit 3 ) ag2 )                                          as "previews"
            from collection_info c
                     left join nft_direct_sell ds on ds.collection = c.address
                     left join token_usd_prices tup_ds on tup_ds.token = ds.price_token
                     left join nft_auction na on na.collection = c.address
                     left join token_usd_prices tup_na on tup_na.token = na.price_token
            order by {order}
            limit $1 offset $2
            "#
        );

        sqlx::query_as(&query)
            .bind(limit as i64)
            .bind(offset as i64)
            .bind(owners)
            .bind(verified)
            .bind(name)
            .bind(collections)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_roots(&self) -> sqlx::Result<Vec<RootRecord>> {
        sqlx::query_as!(
            RootRecord,
            r#"
            select r.address as "address!", r.code::text as "code!"
            from roots r
            where expiry_date is null
               or now()::timestamp < expiry_date;
            "#
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_collections_simple(
        &self,
        name: Option<&String>,
        verified: Option<&bool>,
        limit: usize,
        offset: usize,
    ) -> sqlx::Result<Vec<NftCollectionSimple>> {
        sqlx::query_as!(
            NftCollectionSimple,
            r#"
            select c.address                                                                      as "address!",
                   c.name,
                   c.description,
                   c.logo,
                   c.verified                                                                     as "verified!",
                   count(1) over ()                                                               as "cnt!",
                   ( select count(*) from nft n where n.collection = c.address and not n.burned ) as "nft_count!"
            from nft_collection c
            where ($3::boolean is false or c.verified is true)
              and ($4::varchar is null or c.name ilike $4)
            order by ( select count(*)
                       from ( select distinct owner from nft n where n.collection = c.address and not n.burned ) owners ) desc
            limit $1 offset $2;
            "#,
            limit as i64,
            offset as i64,
            verified,
            name,
        )
        .fetch_all(self.db.as_ref())
        .await
    }
}
