use crate::db::Queries;
use crate::handlers::calculate_hash;
use crate::model::MetricsSummaryBase;
use crate::{catch_error, response};
use chrono::NaiveDateTime;
use moka::future::Cache;
use opg::OpgModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Filter;

#[derive(Debug, Clone, Deserialize, Serialize, Hash, OpgModel)]
pub struct MetricsSummaryQuery {
    pub from: i64,
    pub to: i64,
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
    let mut query = query;
    query.from = (query.from / 300) * 300;
    query.to = (query.to / 300) * 300;

    let hash = calculate_hash(&query);
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
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }
    response!(response)
}
