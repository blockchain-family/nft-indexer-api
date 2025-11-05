pub mod collections;
pub mod metadata;

use std::fmt::Display;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::handlers::requests::collections::CollectionOrderingFields;

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderDirection::Asc => write!(f, "asc"),
            OrderDirection::Desc => write!(f, "desc"),
        }
    }
}

#[derive(Clone, Hash, Deserialize, ToSchema)]
pub struct Ordering<T> {
    pub direction: OrderDirection,
    pub field: T,
}

pub type CollectionListOrder = Ordering<CollectionOrderingFields>;

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FromTo<T> {
    pub from: Option<T>,
    pub to: Option<T>,
}

pub type Period = FromTo<i64>;
