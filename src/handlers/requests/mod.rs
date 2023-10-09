pub mod collections;
pub mod metadata;

use crate::handlers::requests::collections::CollectionOrderingFields;
use serde::Deserialize;
use std::hash::Hash;
use utoipa::ToSchema;

#[derive(Clone, Hash, Deserialize, ToSchema)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Clone, Hash, Deserialize, ToSchema)]
#[aliases(CollectionListOrder = Ordering<CollectionOrderingFields>)]
pub struct Ordering<T> {
    pub direction: OrderDirection,
    pub field: T,
}
