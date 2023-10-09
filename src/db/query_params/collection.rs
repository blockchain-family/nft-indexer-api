use crate::db::query_params::Ordering;
use crate::db::Address;

pub enum CollectionOrderingFields {
    OwnersCount,
    FirstMint,
    FloorPriceUsd,
}

impl From<crate::handlers::requests::collections::CollectionOrderingFields>
    for CollectionOrderingFields
{
    fn from(value: crate::handlers::requests::collections::CollectionOrderingFields) -> Self {
        match value {
            crate::handlers::requests::collections::CollectionOrderingFields::OwnersCount => {
                Self::OwnersCount
            }
            crate::handlers::requests::collections::CollectionOrderingFields::FirstMint => {
                Self::FirstMint
            }
            crate::handlers::requests::collections::CollectionOrderingFields::Price => {
                Self::FloorPriceUsd
            }
        }
    }
}

impl std::fmt::Display for CollectionOrderingFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionOrderingFields::OwnersCount => write!(f, "owners_count"),
            CollectionOrderingFields::FirstMint => write!(f, "first_mint"),
            CollectionOrderingFields::FloorPriceUsd => write!(f, "floor_price_usd"),
        }
    }
}

pub struct CollectionsListParams<'a> {
    pub name: Option<&'a String>,
    pub owners: &'a [String],
    pub verified: Option<bool>,
    pub collections: &'a [Address],
    pub limit: usize,
    pub offset: usize,
    pub order: Option<Ordering<CollectionOrderingFields>>,
    pub nft_type: Option<&'a String>,
}
