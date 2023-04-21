use crate::db::queries::Queries;
use crate::db::Address;
use crate::model::OrderDirection;
use crate::model::{Collection, CollectionDetails, CollectionSimple, VecWithTotal};
use crate::schema::VecCollectionSimpleWithTotal;
use crate::schema::VecCollectionsWithTotal;
use crate::{api_doc_addon, catch_empty, catch_error_500, response};
use serde::{Deserialize, Serialize};
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
        OwnerParam
    )),
    tags(
        (name = "collection", description = "Collection handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Clone, Deserialize, Serialize, ToSchema)]
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

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct CollectionListOrder {
    pub field: CollectionListOrderField,
    pub direction: OrderDirection,
}

#[derive(Clone, Deserialize, ToSchema)]
pub struct ListCollectionsParams {
    pub name: Option<String>,
    pub owners: Option<Vec<String>>,
    pub verified: Option<bool>,
    pub collections: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order: Option<CollectionListOrder>,
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
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections")
        .and(warp::post())
        .and(warp::body::json::<ListCollectionsParams>())
        .and(warp::any().map(move || db.clone()))
        .and_then(list_collections_handler)
}

pub async fn list_collections_handler(
    params: ListCollectionsParams,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owners = params.owners.as_deref().unwrap_or(&[]);
    let verified = Some(params.verified.unwrap_or(true));
    let name = params.name.as_ref();
    let collections = params.collections.as_deref().unwrap_or(&[]);
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();

    let list = catch_error_500!(
        db.list_collections(
            name,
            owners,
            verified.as_ref(),
            collections,
            limit,
            offset,
            params.order,
        )
        .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let mut items: Vec<CollectionDetails> = vec![];
    for collection_detail in list {
        let detail = catch_error_500!(CollectionDetails::from_db(collection_detail));
        items.push(detail);
    }
    let ret = VecWithTotal { count, items };
    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CollectionParam {
    pub collection: Address,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
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
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections" / "simple")
        .and(warp::post())
        .and(warp::body::json::<ListCollectionsSimpleParams>())
        .and(warp::any().map(move || db.clone()))
        .and_then(list_collections_simple_handler)
}

pub async fn list_collections_simple_handler(
    params: ListCollectionsSimpleParams,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let verified = Some(params.verified.unwrap_or(true));
    let name = params.name.as_ref();
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_collections_simple(name, verified.as_ref(), limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<CollectionSimple> = list.into_iter().map(CollectionSimple::from_db).collect();

    let ret = VecWithTotal { count, items: ret };
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
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collection" / "details")
        .and(warp::post())
        .and(warp::body::json::<CollectionParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_collection_handler)
}

pub async fn get_collection_handler(
    param: CollectionParam,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let col = catch_error_500!(db.get_collection(&param.collection).await);
    let col = catch_empty!(col, "");
    let ret = catch_error_500!(CollectionDetails::from_db(col));
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
