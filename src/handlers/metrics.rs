use crate::db::Queries;
use crate::model::MetricsSummaryBase;
use crate::{catch_error, response};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use moka::future::Cache;
use serde_json::Value;
use warp::http::StatusCode;
use warp::Filter;
use crate::handlers::calculate_hash;

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MetricsSummaryQuery {
    pub from: i64,
    pub to: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash)]
pub struct MetricsSummaryQueryCache {
    pub name: String,
    pub limit: i64,
    pub offset: i64,
}

/// GET /metrics/summary
pub fn get_metrics_summary(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("metrics" / "summary")
        .and(warp::get())
        .and(warp::query::<MetricsSummaryQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(metrics_summary_handler)
}

pub async fn metrics_summary_handler(
    query: MetricsSummaryQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let query_cache = MetricsSummaryQueryCache{
        name: "MetricsSummaryQueryCache".to_string(),
        limit: query.limit,
        offset: query.offset
    };
    let hash = calculate_hash(&query_cache);
    let cached_value = cache.get(&hash);

    let response;
    match cached_value {
        None => {
            let from = NaiveDateTime::from_timestamp(query.from, 0);
            let to = NaiveDateTime::from_timestamp(query.to, 0);
            let values = catch_error!(
                db.get_metrics_summary(from, to, query.limit, query.offset)
                    .await
            );
            response = MetricsSummaryBase::from(values);
            let value_for_cache = serde_json::to_value(response.clone()).unwrap();
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => response = serde_json::from_value(cached_value).unwrap(),
    }
    response!(response)
}
