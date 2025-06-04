use super::HttpState;
use crate::db::NftEventType;
use crate::handlers::calculate_hash;
use crate::model::NftEvents;
use crate::{catch_error_500, model::SearchResult, response};
use axum::body::Bytes;
use axum::extract::{Json, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

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
pub async fn search_all(State(s): State<Arc<HttpState>>, query: Bytes) -> impl IntoResponse {
    let query = String::from_utf8(query.into()).expect("err converting to String");
    let items = catch_error_500!(s.db.search_all(&query).await);
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
pub async fn get_events(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<EventsQuery>,
) -> impl IntoResponse {
    let hash = calculate_hash(&query);
    let cached_value = s.cache_10_sec.get(&hash).await;

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

            let verified = if nft.is_some() { Some(false) } else { verified };

            let record = catch_error_500!(
                s.db.list_events(
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
            s.cache_10_sec.insert(hash, value_for_cache).await;
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
