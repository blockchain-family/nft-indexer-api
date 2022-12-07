use sqlx::types::BigDecimal;
use chrono::NaiveDateTime;
use super::*;

pub type Address = String;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchResult {
    pub address: Address,
    pub typ: EventCategory,
    pub name: Option<String>,
    pub nft: Option<Address>,
    pub collection: Option<Address>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Event {
    pub id: i64,
    pub address: String,
    pub event_cat: EventCategory,
    pub event_type: EventType,
    pub created_at: i64,
    pub created_lt: i64,
    pub args: Option<serde_json::Value>,
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
    pub nft_count: Option<i64>,
    pub max_price: Option<BigDecimal>,
    pub total_price: Option<BigDecimal>,
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
}

#[derive(Clone, Debug)]
pub struct NftPrice {
    pub ts: Option<NaiveDateTime>,
    pub usd_price: Option<BigDecimal>,
    pub count: Option<i64>,
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
    pub collection: Option<Address>,
    pub trait_type: String,
    pub values: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct TokenUsdPrice {
    pub token: Address,
    pub usd_price: BigDecimal,
    pub ts: NaiveDateTime,
}

#[derive(Clone, Debug)]
pub struct MetaParsed {
    pub image: Option<String>,
    pub mimetype: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub typ: Option<String>
}

impl NftDetails {
    pub fn parse_meta(&self) -> MetaParsed {
        let meta_obj = match &self.meta {
            Some(meta) if meta.is_object() => meta.as_object().unwrap().clone(),
            _ => serde_json::Map::default(),
        };
        let attributes = meta_obj.get("attributes").map(|a| a.clone());
        let typ = meta_obj.get("type").map(|a| a.as_str().unwrap_or_default().to_string());
        let mut image = meta_obj.get("image").map(|i| i.to_string());
        let mut mimetype: Option<String> = None;
        if image.is_none() {
            // https://github.com/nftalliance/docs/blob/main/src/standard/TIP-4/2.md
            image = meta_obj.get("preview").and_then(|p| {
                p.as_object().and_then(|o| 
                    o.get("source").map(|x| x.as_str().unwrap_or_default().to_string())
                )
            });
            mimetype = meta_obj.get("preview").and_then(|p| {
                p.as_object().and_then(|o| 
                    o.get("mimetype").map(|x| x.as_str().unwrap_or_default().to_string())
                )
            });
        }
        MetaParsed { image, mimetype, attributes, typ }
    }
}

