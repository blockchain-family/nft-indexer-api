use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use crate::model::{
    Auction, AuctionBid, Collection, CollectionDetails, CollectionSimple, DirectBuy, DirectSell,
    NFT,
};
#[derive(ToSchema, Serialize)]
pub struct Address(String);

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VecWithAuction {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<Auction>,
    pub nft: Option<HashMap<Address, NFT>>,
    pub collection: Option<HashMap<Address, Collection>>,
    pub auction: Option<HashMap<Address, Auction>>,
    pub direct_buy: Option<HashMap<Address, DirectBuy>>,
    pub direct_sell: Option<HashMap<Address, DirectSell>>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VecWithAuctionBids {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<AuctionBid>,
    pub nft: Option<HashMap<Address, NFT>>,
    pub collection: Option<HashMap<Address, Collection>>,
    pub auction: Option<HashMap<Address, Auction>>,
    pub direct_buy: Option<HashMap<Address, DirectBuy>>,
    pub direct_sell: Option<HashMap<Address, DirectSell>>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VecWithDirectBuy {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<DirectBuy>,
    pub nft: Option<HashMap<String, NFT>>,
    pub collection: Option<HashMap<String, Collection>>,
    pub auction: Option<HashMap<String, Auction>>,
    pub direct_buy: Option<HashMap<String, DirectBuy>>,
    pub direct_sell: Option<HashMap<String, DirectSell>>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VecWithDirectSell {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<DirectSell>,
    pub nft: Option<HashMap<String, NFT>>,
    pub collection: Option<HashMap<String, Collection>>,
    pub auction: Option<HashMap<String, Auction>>,
    pub direct_buy: Option<HashMap<String, DirectBuy>>,
    pub direct_sell: Option<HashMap<String, DirectSell>>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VecWithNFT {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<NFT>,
    pub nft: Option<HashMap<String, NFT>>,
    pub collection: Option<HashMap<String, Collection>>,
    pub auction: Option<HashMap<String, Auction>>,
    pub direct_buy: Option<HashMap<String, DirectBuy>>,
    pub direct_sell: Option<HashMap<String, DirectSell>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VecCollectionsWithTotal {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<CollectionDetails>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VecCollectionSimpleWithTotal {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<CollectionSimple>,
}
