use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::HttpState;
use crate::handlers::calculate_hash;
use crate::model::MetricsSummaryBase;
use crate::{catch_error_500, response};

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
pub async fn get_metrics_summary(
    State(s): State<Arc<HttpState>>,
    Query(query): Query<MetricsSummaryQuery>,
) -> impl IntoResponse {
    let mut query = query;
    query.from = (query.from / 300) * 300;
    query.to = (query.to / 300) * 300;

    let hash = calculate_hash(&query);
    let cached_value = s.cache_minute.get(&hash).await;

    let response;
    match cached_value {
        None => {
            let from = DateTime::from_timestamp(query.from, 0)
                .expect("Failed to get datetime")
                .naive_utc();
            let to = DateTime::from_timestamp(query.to, 0)
                .expect("Failed to get datetime")
                .naive_utc();
            let values = catch_error_500!(
                s.db.get_metrics_summary(from, to, query.limit, query.offset)
                    .await
            );
            response = MetricsSummaryBase::from(values);
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            s.cache_minute.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }
    response!(response)
}
