use std::{convert::Infallible, collections::HashMap};
use serde::Deserialize;
use warp::http::StatusCode;
use crate::db::{Address, Queries};
use warp::Filter;
use crate::model::{Collection, VecWithTotal};


#[derive(Debug, Clone, Deserialize)]
pub struct ListCollectionsParams {
    pub name: Option<String>,
    pub owners: Option<Vec<String>>,
    pub verified: Option<bool>,
    pub collections: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// POST /collections
pub fn list_collections(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("collections")
        .and(warp::post())
        .and(warp::body::json::<ListCollectionsParams>())
        .and(warp::any().map(move || db.clone()))
        .and_then(list_collections_handler)
}

pub async fn list_collections_handler(params: ListCollectionsParams, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owners = params.owners.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let verified = Some(params.verified.clone().unwrap_or(true));
    let name = params.name.as_ref();
    let collections = params.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();
    let count = match db.list_collections_count(name, owners, verified.as_ref(), collections).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_collections(name, owners, verified.as_ref(), collections, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<Collection> = list.iter().map(|c| Collection::from_db(c, &db.tokens)).collect();
            let ret = VecWithTotal { count, items: ret };
            Ok(Box::from(
                warp::reply::with_status(
                    warp::reply::json(&ret), 
                    StatusCode::OK)
            ))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollectionParam {
    pub collection: Address,
}

/// POST /collection/details
pub fn get_collection(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("collection" / "details")
        .and(warp::post())
        .and(warp::body::json::<CollectionParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_collection_handler)
}

pub async fn get_collection_handler(param: CollectionParam, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    match db.get_collection((&param.collection).into()).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(None) => Ok(Box::from(warp::reply::with_status(String::default(), StatusCode::BAD_REQUEST))),
        Ok(Some(col)) => {
            let ret = Collection::from_db(&col, &db.tokens);
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OwnerParam {
    pub owner: Address,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// POST /collections/by-owner
pub fn get_collections_by_owner(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("collections" / "by-owner")
        .and(warp::post())
        .and(warp::body::json::<OwnerParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_collections_by_owner_handler)
}

pub async fn get_collections_by_owner_handler(params: OwnerParam, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owner = params.owner;
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();
    let count = match db.list_collections_by_owner_count(&owner).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_collections_by_owner(&owner, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<Collection> = list.iter().map(|col| Collection::from_db(col, &db.tokens)).collect();
            let ret = VecWithTotal { count, items: ret };
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

pub async fn collect_collections(db: &Queries, ids: &Vec<String>) -> anyhow::Result<HashMap<String, Collection>> {
    let dblist = db.collect_collections(ids).await?;
    let list = dblist
        .iter()
        .map(|col| Collection::from_db(col, &db.tokens));
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.contract.address.clone(), item.clone());
    }
    Ok(map)
}