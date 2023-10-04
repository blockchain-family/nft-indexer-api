pub mod collections;
pub mod metadata;

use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Clone, Hash, Deserialize, ToSchema)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Clone, Hash, Deserialize, ToSchema)]
pub struct Ordering<T> {
    pub direction: OrderDirection,
    pub field: T,
}
