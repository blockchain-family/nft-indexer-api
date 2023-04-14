use crate::db::Queries;
use crate::model::MetricsSummaryBase;
use crate::{catch_error_500, response};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Filter;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsSummaryQuery {
    pub from: i64,
    pub to: i64,
    pub limit: i64,
    pub offset: i64,
}

/// GET /metrics/summary
pub fn get_metrics_summary(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("metrics" / "summary")
        .and(warp::get())
        .and(warp::query::<MetricsSummaryQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(metrics_summary_handler)
}

pub async fn metrics_summary_handler(
    query: MetricsSummaryQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let from = NaiveDateTime::from_timestamp_opt(query.from, 0).expect("Failed to get datetime");
    let to = NaiveDateTime::from_timestamp_opt(query.to, 0).expect("Failed to get datetime");
    let values = catch_error_500!(
        db.get_metrics_summary(from, to, query.limit, query.offset)
            .await
    );
    response!(&MetricsSummaryBase::from(values))
}
