use std::convert::Infallible;
use warp::http::StatusCode;
use crate::{db::Queries, model::SearchResult};
use warp::Filter;
use crate::model::NftEvent;
use serde::{Serialize, Deserialize};
use warp::hyper::body::Bytes;
use crate::db::{NftEventCategory, NftEventType};


/// POST /search
pub fn search_all(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("search")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::any().map(move || db.clone()))
        .and_then(search_all_handler)
}

pub async fn search_all_handler(query: Bytes, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let query = String::from_utf8(query.into()).expect("err converting to String");
    let items: Vec<SearchResult> = match db.search_all(&query).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(ref xs) => xs.iter().map(|x| SearchResult::from_db(x)).collect(),
    };
    let count = items.len();
    Ok(Box::from(
        warp::reply::with_status(
            warp::reply::json(&SearchRes{ items, count }), 
            StatusCode::OK)
    ))
}

/// POST /events
pub fn get_events(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(warp::body::json::<EventsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_events_handler)
}

pub async fn get_events_handler(query: EventsQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let nft = query.nft.as_ref();
    let collection = query.collection.as_ref();
    let event_type = query.event_type.as_deref().unwrap_or(&[]);
    let category = query.category.as_deref().unwrap_or(&[]);
    let owner = query.owner.as_ref();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    match db.list_events(nft, collection, owner, event_type, category, offset, limit).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let response: Result<Vec<NftEvent>, serde_json::Error> = match list.events {
                None => Ok(vec![]),
                Some(value) => {
                    serde_json::from_value(value)
                }
            };

            match response {
                Ok(response) => {
                    Ok(Box::from(
                        warp::reply::with_status(
                            warp::reply::json(&response),
                            StatusCode::OK)
                    ))
                }
                Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)))
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventsQuery {
    pub owner: Option<String>,
    pub collection: Option<String>,
    pub nft: Option<String>,
    pub category: Option<Vec<NftEventCategory>>,
    #[serde(rename = "type")]
    pub event_type: Option<Vec<NftEventType>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchRes {
    pub items: Vec<SearchResult>,
    pub count: usize,
}
