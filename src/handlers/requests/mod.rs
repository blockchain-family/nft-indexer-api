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

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, Deserialize, ToSchema)]
#[aliases(Period = FromTo<i64>)]
#[serde(rename_all = "camelCase")]
pub struct FromTo<T> {
    pub from: Option<T>,
    pub to: Option<T>,
}
