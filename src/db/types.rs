use super::*;
use chrono::NaiveDateTime;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use utoipa::ToSchema;
pub type Address = String;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchResult {
    pub address: Address,
    pub object_type: String,
    pub nft_name: Option<String>,
    pub collection_name: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NftDetails {
    pub address: Option<String>,
    pub collection: Option<Address>,
    pub meta: Option<serde_json::Value>,
    pub owner: Option<Address>,
    pub manager: Option<Address>,
    pub name: Option<String>,
    pub burned: Option<bool>,
    pub description: Option<String>,
    pub updated: Option<NaiveDateTime>,
    pub tx_lt: Option<i64>,
    pub auction: Option<String>,
    pub forsale: Option<String>,
    pub auction_status: Option<AuctionStatus>,
    pub forsale_status: Option<DirectSellState>,
    pub best_offer: Option<String>,
    pub floor_price_usd: Option<BigDecimal>,
    pub deal_price_usd: Option<BigDecimal>,
    pub total_count: i64,
    pub floor_price: Option<BigDecimal>,
    pub floor_price_token: Option<Address>,
    pub nft_id: Option<Address>,
}

#[derive(Clone, Debug)]
pub struct Nft {
    pub address: Address,
    pub collection: Option<Address>,
    pub owner: Option<Address>,
    pub manager: Option<Address>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub burned: bool,
    pub updated: NaiveDateTime,
    pub tx_lt: i64,
}

#[derive(Clone, Debug)]
pub struct NftMeta {
    pub nft: Address,
    pub meta: serde_json::Value,
    pub updated: NaiveDateTime,
}

#[derive(Clone, Debug)]
pub struct NftCollection {
    pub address: Address,
    pub owner: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub verified: bool,
    pub wallpaper: Option<String>,
    pub logo: Option<String>,
    pub owners_count: Option<i32>,
    pub nft_count: i64,
    pub max_price: Option<BigDecimal>,
    pub total_price: Option<BigDecimal>,
    pub cnt: i64,
    pub first_mint: NaiveDateTime,
}

#[derive(Clone, Debug)]
pub struct NftCollectionSimple {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub verified: bool,
    pub logo: Option<String>,
    pub cnt: i64,
    pub nft_count: i64,
}

#[derive(Clone, Debug)]
pub struct RootRecord {
    pub address: Address,
    pub code: String,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct NftCollectionDetails {
    pub address: Option<Address>,
    pub owner: Option<Address>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
    pub verified: Option<bool>,
    pub wallpaper: Option<String>,
    pub logo: Option<String>,
    pub owners_count: Option<i64>,
    pub nft_count: Option<i64>,
    pub max_price: Option<BigDecimal>,
    pub total_price: Option<BigDecimal>,
    pub floor_price_usd: Option<BigDecimal>,
    pub total_volume_usd: Option<BigDecimal>,
    pub attributes: Option<serde_json::Value>,
    pub cnt: i64,
    pub previews: serde_json::Value,
    pub first_mint: Option<NaiveDateTime>,
}

#[derive(Clone, Debug)]
pub struct NftAuction {
    pub address: Option<Address>,
    pub nft: Option<Address>,
    pub wallet_for_bids: Option<Address>,
    pub price_token: Option<Address>,
    pub start_price: Option<BigDecimal>,
    pub max_bid: Option<BigDecimal>,
    pub min_bid: Option<BigDecimal>,
    pub start_usd_price: Option<BigDecimal>,
    pub max_usd_bid: Option<BigDecimal>,
    pub min_usd_bid: Option<BigDecimal>,
    pub status: Option<AuctionStatus>,
    pub created_at: Option<NaiveDateTime>,
    pub finished_at: Option<NaiveDateTime>,
    pub tx_lt: Option<i64>,
    pub bids_count: Option<i64>,
    pub last_bid_from: Option<Address>,
    pub last_bid_ts: Option<NaiveDateTime>,
    pub last_bid_value: Option<BigDecimal>,
    pub last_bid_usd_value: Option<BigDecimal>,
    pub cnt: i64,
    pub fee_numerator: Option<i32>,
    pub fee_denominator: Option<i32>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct NftTraitRecord {
    pub trait_type: Option<String>,
    pub trait_value: Option<String>,
    pub cnt: i64,
}

#[derive(Clone, Debug)]
pub struct NftAuctionBid {
    pub auction: Address,
    pub buyer: Address,
    pub price: BigDecimal,
    pub usd_price: Option<BigDecimal>,
    pub next_bid_value: BigDecimal,
    pub next_bid_usd_value: Option<BigDecimal>,
    pub created_at: NaiveDateTime,
    pub tx_lt: i64,
    pub active: bool,
    pub cnt: i64,
}

#[derive(Clone, Debug)]
pub struct NftAuctionBidExt {
    pub auction: Address,
    pub buyer: Address,
    pub price_token: Option<Address>,
    pub price: BigDecimal,
    pub usd_price: Option<BigDecimal>,
    pub next_bid_value: Option<BigDecimal>,
    pub next_bid_usd_value: Option<BigDecimal>,
    pub created_at: NaiveDateTime,
    pub tx_lt: Option<i64>,
    pub active: Option<bool>,
    pub nft: Option<Address>,
    pub collection: Option<Address>,
    pub cnt: i64,
}

#[derive(Clone, Debug)]
pub struct NftDirectSell {
    pub address: Address,
    pub nft: Address,
    pub collection: Option<Address>,
    pub seller: Option<Address>,
    pub price_token: Address,
    pub price: BigDecimal,
    pub usd_price: Option<BigDecimal>,
    pub state: DirectSellState,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
    pub tx_lt: i64,
    pub cnt: i64,
    pub fee_numerator: Option<i32>,
    pub fee_denominator: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct NftDirectBuy {
    pub address: Address,
    pub nft: Address,
    pub collection: Option<Address>,
    pub buyer: Option<Address>,
    pub price_token: Address,
    pub price: BigDecimal,
    pub usd_price: Option<BigDecimal>,
    pub state: DirectBuyState,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
    pub tx_lt: i64,
    pub cnt: i64,
    pub fee_numerator: Option<i32>,
    pub fee_denominator: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct NftPriceHistory {
    pub source: Address,
    pub source_type: NftPriceSource,
    pub ts: NaiveDateTime,
    pub price: BigDecimal,
    pub price_token: Option<Address>,
    pub nft: Option<Address>,
    pub collection: Option<Address>,
    pub is_deal: bool,
}

#[derive(Clone, Debug)]
pub struct NftPrice {
    pub ts: NaiveDateTime,
    pub usd_price: BigDecimal,
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub address: Address,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub email: Option<String>,
    pub site: Option<String>,
    pub twitter: Option<String>,
    pub created: NaiveDateTime,
}

#[derive(Clone, Debug)]
pub struct TraitDef {
    pub collection: Address,
    pub trait_type: String,
    pub values: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct TokenUsdPrice {
    pub token: Address,
    pub usd_price: BigDecimal,
    pub ts: NaiveDateTime,
}

#[derive(Clone, Debug, Default)]
pub struct MetaParsed {
    pub image: Option<String>,
    pub mimetype: Option<String>,
    pub full_image: Option<String>,
    pub full_image_mimetype: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub typ: Option<String>,
    pub royalty: Option<MetaRoyalty>,
}
#[derive(Deserialize, Clone, Debug)]
struct MetaFile {
    pub source: Option<String>,
    pub mimetype: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MetaRoyalty {
    pub description: Option<String>,
    pub royalty_type: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct MetaJson {
    pub files: Vec<MetaFile>,
    pub preview: Option<MetaFile>,
    #[serde(rename = "type")]
    pub typ: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub royalty: Option<MetaRoyalty>,
}

impl NftDetails {
    pub fn parse_meta(&self) -> MetaParsed {
        match self.meta.clone() {
            Some(meta) => {
                let meta_json = serde_json::from_value::<MetaJson>(meta);
                match meta_json {
                    Ok(meta_json) => {
                        let typ = meta_json.typ;
                        let preview = meta_json
                            .preview
                            .map_or((None, None), |p| (p.source, p.mimetype));
                        let full_image = meta_json
                            .files
                            .first()
                            .map_or((None, None), |p| (p.source.clone(), p.mimetype.clone()));

                        MetaParsed {
                            image: preview.0,
                            mimetype: preview.1,
                            full_image: full_image.0,
                            full_image_mimetype: full_image.1,
                            attributes: meta_json.attributes,
                            typ,
                            royalty: meta_json.royalty,
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse meta {:#?} {e}", self.meta);
                        MetaParsed::default()
                    }
                }
            }
            None => MetaParsed::default(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow)]
pub struct NftEventsRecord {
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct MetricsSummaryRecord {
    pub collection: String,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub floor_price: Option<BigDecimal>,
    pub total_volume_usd_now: BigDecimal,
    pub total_volume_usd_previous: BigDecimal,
    pub owners_count: i32,
    pub nfts_count: i32,
    pub total_rows_count: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct OwnerFeeRecord {
    pub fee_numerator: i32,
    pub fee_denominator: i32,
    pub collection: Option<String>,
    pub nft: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Default)]
pub struct UserRecord {
    pub address: String,
    pub logo_nft: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub twitter: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub link: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}
