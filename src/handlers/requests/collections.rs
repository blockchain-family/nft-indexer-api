use crate::handlers::requests::CollectionListOrder;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Hash, ToSchema)]
pub enum CollectionOrderingFields {
    #[serde(rename = "ownersCount")]
    OwnersCount,
    #[serde(rename = "firstMint")]
    FirstMint,
    #[serde(rename = "price")]
    Price,
}

#[derive(Clone, Deserialize, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListCollectionsParams {
    pub name: Option<String>,
    pub owners: Option<Vec<String>>,
    pub verified: Option<bool>,
    pub collections: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order: Option<CollectionListOrder>,
    pub nft_types: Option<Vec<String>>,
}
