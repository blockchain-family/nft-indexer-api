use crate::db::{NftEventCategory, NftEventType};
use crate::model::NftEvents;
use crate::{catch_error, db::Queries, model::SearchResult, response};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::Filter;

/// POST /search
pub fn search_all(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("search")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::any().map(move || db.clone()))
        .and_then(search_all_handler)
}

pub async fn search_all_handler(
    query: Bytes,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let query = String::from_utf8(query.into()).expect("err converting to String");

    let items = catch_error!(db.search_all(&query).await);
    let items: Vec<SearchResult> = items.into_iter().map(SearchResult::from_db).collect();
    let count = items.len();
    response!(&SearchRes { items, count })
}

/// POST /events
pub fn get_events(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(warp::body::json::<EventsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_events_handler)
}

pub async fn get_events_handler(
    query: EventsQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let nft = query.nft.as_ref();
    let event_type = query.event_type.as_deref().unwrap_or(&[]);
    let collection = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner.as_ref();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let with_count = query.with_count.unwrap_or(false);
    let verified = query.verified;

    let final_limit = match with_count {
        true => limit,
        false => limit + 1,
    };
    let record = catch_error!(
        db.list_events(
            nft,
            collection,
            owner,
            event_type,
            offset,
            final_limit,
            with_count,
            verified
        )
        .await
    );

    let response: Result<NftEvents, serde_json::Error> = match record.content {
        None => Ok(NftEvents::default()),
        Some(value) => serde_json::from_value(value),
    };

    let mut response = catch_error!(response);

    if !with_count {
        if response.data.len() < final_limit {
            response.total_rows = (response.data.len() + offset) as i64
        } else {
            response.data.pop();
            response.total_rows = (response.data.len() + offset + 1) as i64;
        }
    }
    response!(&response)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventsQuery {
    pub owner: Option<String>,
    pub collections: Option<Vec<String>>,
    pub nft: Option<String>,
    pub categories: Option<Vec<NftEventCategory>>,
    #[serde(rename = "types")]
    pub event_type: Option<Vec<NftEventType>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "withCount")]
    pub with_count: Option<bool>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchRes {
    pub items: Vec<SearchResult>,
    pub count: usize,
}
