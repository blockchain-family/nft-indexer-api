use crate::db::queries::Queries;
use crate::db::NftEventType;
use crate::handlers::calculate_hash;
use crate::model::AuctionActive;
use crate::model::AuctionBidPlaced;
use crate::model::AuctionCanceled;
use crate::model::AuctionComplete;
use crate::model::NftEvent;
use crate::model::NftEventAuction;
use crate::model::NftEventDirectBuy;
use crate::model::NftEventDirectSell;
use crate::model::NftEventMint;
use crate::model::NftEventTransfer;
use crate::model::NftEvents;
use crate::{api_doc_addon, catch_error_500, model::SearchResult, response};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(search_all, get_events),
    components(schemas(
        SearchResult,
        SearchRes,
        EventsQuery,
        NftEvents,
        NftEvent,
        NftEventDirectSell,
        NftEventDirectBuy,
        NftEventAuction,
        NftEventMint,
        NftEventTransfer,
        AuctionActive,
        AuctionComplete,
        AuctionCanceled,
        AuctionBidPlaced
    )),
    tags(
        (name = "event", description = "Event handlers"),
    ),
)]

struct ApiDoc;
api_doc_addon!(ApiDoc);

#[utoipa::path(
    post,
    tag = "event",
    path = "/search",
    request_body(content = String, description = "Search events"),
    responses(
        (status = 200, body = SearchRes),
        (status = 500),
    ),
)]
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
    let items = catch_error_500!(db.search_all(&query).await);
    let items: Vec<SearchResult> = items.into_iter().map(SearchResult::from_db).collect();
    let count = items.len();
    response!(&SearchRes { items, count })
}

#[utoipa::path(
    post,
    tag = "event",
    path = "/events",
    request_body(content = EventsQuery, description = "List events"),
    responses(
        (status = 200, body = NftEvents),
        (status = 500),
    ),
)]
pub fn get_events(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("events")
        .and(warp::post())
        .and(warp::body::json::<EventsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_events_handler)
}

pub async fn get_events_handler(
    query: EventsQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&query);
    let cached_value = cache.get(&hash);

    let response;
    match cached_value {
        None => {
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
            let record = catch_error_500!(
                db.list_events(
                    nft,
                    collection,
                    owner,
                    event_type,
                    offset,
                    final_limit,
                    with_count,
                    verified,
                )
                .await
            );

            let r: Result<NftEvents, serde_json::Error> = match record.content {
                None => Ok(NftEvents::default()),
                Some(value) => serde_json::from_value(value),
            };

            let mut r = catch_error_500!(r);

            if !with_count {
                if r.data.len() < final_limit {
                    r.total_rows = (r.data.len() + offset) as i64
                } else {
                    r.data.pop();
                    r.total_rows = (r.data.len() + offset + 1) as i64;
                }
            }

            response = r;
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&response)
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct EventsQuery {
    pub owner: Option<String>,
    pub collections: Option<Vec<String>>,
    pub nft: Option<String>,
    #[serde(rename = "types")]
    pub event_type: Option<Vec<NftEventType>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "withCount")]
    pub with_count: Option<bool>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SearchRes {
    pub items: Vec<SearchResult>,
    pub count: usize,
}
