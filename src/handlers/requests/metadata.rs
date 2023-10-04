use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, Hash, ToSchema)]
pub struct UpdateMetadataParams {
    pub nft: Option<String>,
    pub collection: String,
}
