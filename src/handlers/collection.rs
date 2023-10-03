use crate::db::queries::Queries;
use crate::db::query_params::collection::CollectionsListParams;
use crate::db::Address;
use crate::handlers::{
    calculate_hash, requests::collections::ListCollectionsParams, OrderDirection,
};
use crate::model::{Collection, CollectionDetails, CollectionSimple, VecWithTotal};
use crate::schema::VecCollectionSimpleWithTotal;
use crate::schema::VecCollectionsWithTotal;
use crate::{api_doc_addon, catch_empty, catch_error_500, response};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Display;
use std::{collections::HashMap, convert::Infallible};
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_collections,
        list_collections_simple,
        get_collection,
        get_collections_by_owner
    ),
    components(schemas(
        CollectionListOrderField,
        CollectionListOrder,
        VecCollectionsWithTotal,
        ListCollectionsParams,
        ListCollectionsSimpleParams,
        VecCollectionSimpleWithTotal,
        CollectionParam,
        CollectionSimple,
        OwnerParam,
        CollectionParam
    )),
    tags(
        (name = "collection", description = "Collection handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub enum CollectionListOrderField {
    #[serde(rename = "firstMint")]
    FirstMint,
}

impl Display for CollectionListOrderField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionListOrderField::FirstMint => write!(f, "first_mint"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct CollectionListOrder {
    pub field: CollectionListOrderField,
    pub direction: OrderDirection,
}

#[utoipa::path(
    post,
    tag = "collection",
    path = "/collections",
    request_body(content = ListCollectionsParams, description = "List collections"),
    responses(
        (status = 200, body = VecCollectionsWithTotal),
        (status = 500),
    ),
)]
pub fn list_collections(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections")
        .and(warp::post())
        .and(warp::body::json::<ListCollectionsParams>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(list_collections_handler)
}

pub async fn list_collections_handler(
    params: ListCollectionsParams,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&params);
    let cached_value = cache.get(&hash);

    let ret: VecWithTotal<CollectionDetails>;
    match cached_value {
        None => {
            let params = CollectionsListParams {
                name: params.name.as_ref(),
                owners: params.owners.as_deref().unwrap_or(&[]),
                verified: Some(params.verified.unwrap_or(true)),
                collections: params.collections.as_deref().unwrap_or(&[]),
                limit: params.limit.unwrap_or(100),
                offset: params.offset.unwrap_or_default(),
                order: params.order.map(|order| order.into()),
                nft_type: params.nft_type.as_ref(),
            };

            let list = db.list_collections(&params).await;

            let list = catch_error_500!(list);

            let count = list.first().map(|it| it.cnt).unwrap_or_default();
            let mut items = vec![];
            for collection_detail in list {
                let detail = catch_error_500!(CollectionDetails::from_db(collection_detail));
                items.push(detail);
            }
            ret = VecWithTotal { count, items };
            let value_for_cache =
                serde_json::to_value(ret.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            ret = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, Hash, ToSchema)]
pub struct CollectionParam {
    pub collection: Address,
}

#[derive(Debug, Clone, Deserialize, Hash, ToSchema)]
pub struct ListCollectionsSimpleParams {
    pub name: Option<String>,
    pub verified: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[utoipa::path(
    post,
    tag = "collection",
    path = "/collections/simple",
    request_body(content = ListCollectionsSimpleParams, description = "List collections simple"),
    responses(
        (status = 200, body = VecCollectionSimpleWithTotal),
        (status = 500),
    ),
)]
pub fn list_collections_simple(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections" / "simple")
        .and(warp::post())
        .and(warp::body::json::<ListCollectionsSimpleParams>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(list_collections_simple_handler)
}

pub async fn list_collections_simple_handler(
    params: ListCollectionsSimpleParams,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&params);
    let cached_value = cache.get(&hash);
    let ret: VecWithTotal<CollectionSimple>;

    match cached_value {
        None => {
            let verified = Some(params.verified.unwrap_or(true));
            let name = params.name.as_ref();
            let limit = params.limit.unwrap_or(100);
            let offset = params.offset.unwrap_or_default();
            let list = catch_error_500!(
                db.list_collections_simple(name, verified.as_ref(), limit, offset)
                    .await
            );
            let count = list.first().map(|it| it.cnt).unwrap_or_default();
            let items = list.into_iter().map(CollectionSimple::from_db).collect();

            ret = VecWithTotal { count, items };
            let value_for_cache =
                serde_json::to_value(ret.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            ret = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&ret)
}

#[utoipa::path(
    post,
    tag = "collection",
    path = "/collection/details",
    request_body(content = CollectionParam, description = "Collection details"),
    responses(
        (status = 200, body = CollectionDetails),
        (status = 500),
    ),
)]
pub fn get_collection(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collection" / "details")
        .and(warp::post())
        .and(warp::body::json::<CollectionParam>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_collection_handler)
}

pub async fn get_collection_handler(
    param: CollectionParam,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&param);
    let cached_value = cache.get(&hash);
    let ret;
    match cached_value {
        None => {
            let col = catch_error_500!(db.get_collection(&param.collection).await);
            let col = catch_empty!(col, "");
            ret = catch_error_500!(CollectionDetails::from_db(col));
            let value_for_cache =
                serde_json::to_value(ret.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            ret = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }
    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct OwnerParam {
    pub owner: Address,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[utoipa::path(
    post,
    tag = "collection",
    path = "/collections/by-owner",
    request_body(content = OwnerParam, description = "Collections by owner"),
    responses(
        (status = 200, body = VecCollectionsWithTotal),
        (status = 500),
    ),
)]
pub fn get_collections_by_owner(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections" / "by-owner")
        .and(warp::post())
        .and(warp::body::json::<OwnerParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_collections_by_owner_handler)
}

pub async fn get_collections_by_owner_handler(
    params: OwnerParam,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owner = params.owner;
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();

    let list = catch_error_500!(db.list_collections_by_owner(&owner, limit, offset).await);

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<Collection> = list.into_iter().map(Collection::from_db).collect();
    let ret = VecWithTotal { count, items: ret };
    response!(&ret)
}

#[allow(clippy::ptr_arg)]
pub async fn collect_collections(
    db: &Queries,
    ids: &Vec<String>,
) -> anyhow::Result<HashMap<String, Collection>> {
    let dblist = db.collect_collections(ids).await?;
    let list = dblist.into_iter().map(Collection::from_db);
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.contract.address.clone(), item.clone());
    }
    Ok(map)
}
