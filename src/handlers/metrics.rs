use crate::db::queries::Queries;
use crate::handlers::calculate_hash;
use crate::model::MetricsSummary;
use crate::model::MetricsSummaryBase;
use crate::{api_doc_addon, catch_error_500, response};
use chrono::NaiveDateTime;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use utoipa::IntoParams;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(get_metrics_summary, ),
    components(schemas(MetricsSummaryBase, MetricsSummary)),
    tags(
        (name = "metrics", description = "Metrics handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, ToSchema, Hash)]
#[into_params(parameter_in = Query)]
pub struct MetricsSummaryQuery {
    pub from: i64,
    pub to: i64,
    pub limit: i64,
    pub offset: i64,
}

#[utoipa::path(
    get,
    tag = "metrics",
    path = "/metrics/summary",
    params(MetricsSummaryQuery),
    responses(
        (status = 200, body = MetricsSummaryBase),
        (status = 500),
    ),
)]
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
    let cached_value = cache.get(&hash).await;

    let response;
    match cached_value {
        None => {
            let from =
                NaiveDateTime::from_timestamp_opt(query.from, 0).expect("Failed to get datetime");
            let to =
                NaiveDateTime::from_timestamp_opt(query.to, 0).expect("Failed to get datetime");
            let values = catch_error_500!(
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
