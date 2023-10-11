use crate::db::{
    MetaRoyalty, MetricsSummaryRecord, NftEventType, NftTraitRecord, NftsPriceRangeRecord,
    OwnerFeeRecord, RootRecord, Social, UserRecord,
};
use crate::{
    db::{Address, AuctionStatus, DirectBuyState, DirectSellState, EventCategory, EventType},
    token::TokenDict,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;

use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecWithTotal<T> {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VecWith<T> {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<T>,
    pub nft: Option<HashMap<Address, NFT>>,
    pub collection: Option<HashMap<Address, Collection>>,
    pub auction: Option<HashMap<Address, Auction>>,
    pub direct_buy: Option<HashMap<Address, DirectBuy>>,
    pub direct_sell: Option<HashMap<Address, DirectSell>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Contract {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: Option<Address>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Price {
    #[serde(rename = "priceToken")]
    pub token: Address,
    pub price: String,
    #[serde(rename = "usdPrice")]
    pub usd_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Fee {
    pub numerator: i32,
    pub denominator: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct NFTPrice {
    #[serde(rename = "usdPrice")]
    pub usd_price: String,
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftTrait {
    pub trait_type: Option<String>,
    #[schema(value_type = String)]
    pub trait_value: Option<Value>,
    pub cnt: i64,
}

impl From<NftTraitRecord> for NftTrait {
    fn from(value: NftTraitRecord) -> Self {
        Self {
            trait_type: value.trait_type,
            trait_value: value.trait_value,
            cnt: value.cnt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NFT {
    #[serde(flatten)]
    pub contract: Contract,
    pub collection: Address,
    pub image: Option<String>,
    pub mimetype: Option<String>,
    pub full_image: Option<String>,
    pub full_image_mimetype: Option<String>,
    #[serde(rename = "type")]
    pub typ: Option<String>,
    pub auction: Option<Address>,
    pub forsale: Option<Address>,
    pub best_offer: Option<Address>,
    pub manager: Option<Address>,
    pub deal_price_usd: Option<String>,
    pub floor_price: Option<Price>,
    pub nft_id: Option<Address>,
    pub royalty: Option<MetaRoyalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(flatten)]
    pub contract: Contract,
    pub verified: Option<bool>,
    pub created_at: usize,
    pub wallpaper: Option<String>,
    pub logo: Option<String>,
    pub owners_count: usize,
    pub nft_count: usize,
    pub lowest_price: Option<String>,
    pub total_price: Option<String>,
    pub first_mint: i64,
    pub social: Option<Social>,
    pub royalty: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CollectionDetailsPreviewMeta {
    pub source: Option<String>,
    pub mimetype: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionDetails {
    #[serde(flatten)]
    pub collection: Collection,
    pub floor_price_usd: Option<String>,
    pub total_volume_usd: Option<String>,
    pub attributes: Vec<Attribute>,
    pub previews: Vec<CollectionDetailsPreviewMeta>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub trait_type: String,
    pub trait_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSimple {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub verified: bool,
    pub logo: Option<String>,
    pub nft_count: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Root {
    pub address: Address,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Roots {
    pub roots: Vec<Root>,
}

impl From<RootRecord> for Root {
    fn from(value: RootRecord) -> Self {
        Self {
            address: value.address,
            code: value.code,
        }
    }
}

impl CollectionSimple {
    pub fn from_db(db: crate::db::NftCollectionSimple) -> Self {
        Self {
            address: db.address,
            name: Some(db.name.unwrap_or_default()),
            description: Some(db.description.unwrap_or_default()),
            verified: db.verified,
            logo: db.logo,
            nft_count: db.nft_count,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: i64,
    #[serde(rename = "type")]
    pub typ: EventType,
    pub cat: EventCategory,
    pub address: String,
    pub ts: usize,
    pub args: Option<Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    pub address: Address,
    pub nft: Address,
    pub status: AuctionStatus,
    pub bid_token: Address,
    pub wallet_for_bids: Option<Address>,
    pub start_bid: Option<String>,
    pub start_usd_bid: Option<String>,
    pub min_bid: Option<String>,
    pub min_usd_bid: Option<String>,
    pub max_bid: Option<String>,
    pub max_usd_bid: Option<String>,
    pub start_time: Option<i64>,
    pub finish_time: Option<i64>,
    pub last_bid_from: Option<Address>,
    #[serde(rename = "lastBidTime")]
    pub last_bid_ts: Option<i64>,
    pub last_bid_value: Option<String>,
    pub last_bid_usd_value: Option<String>,
    pub fee: Fee,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuctionBid {
    pub from: Address,
    pub auction: Address,
    pub nft: Address,
    pub price: String,
    pub usd_price: Option<String>,
    pub active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema, Deserialize)]
pub struct DirectSell {
    pub address: Address,
    pub nft: Address,
    pub seller: Option<Address>,
    pub price: Price,
    pub status: DirectSellState,
    #[serde(rename = "createdAt")]
    pub created: i64,
    #[serde(rename = "finishedAt")]
    pub finished: Option<i64>,
    #[serde(rename = "expiredAt")]
    pub expired: Option<i64>,
    pub fee: Fee,
}

#[derive(Debug, Clone, Serialize, ToSchema, Deserialize)]
pub struct DirectBuy {
    pub address: Address,
    pub nft: Address,
    pub buyer: Option<Address>,
    pub price: Price,
    pub status: DirectBuyState,
    #[serde(rename = "createdAt")]
    pub created: i64,
    #[serde(rename = "finishedAt")]
    pub finished: Option<i64>,
    #[serde(rename = "expiredAt")]
    pub expired: Option<i64>,
    pub fee: Fee,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionAttributes {
    pub collection: Address,
    pub attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub address: Address,
    pub object_type: String,
    pub nft_name: Option<String>,
    pub collection_name: Option<String>,
    pub image: Option<String>,
}

impl CollectionAttributes {
    pub fn from_db(defs: &[crate::db::TraitDef]) -> Vec<Self> {
        let mut res = Vec::with_capacity(16);
        let mut collection = None;
        let mut attributes = HashMap::new();
        for attr in defs.iter() {
            let col = attr.collection.clone();
            match collection.as_ref() {
                None => collection = Some(col),
                Some(c) if *c == col => {
                    attributes.insert(
                        attr.trait_type.clone(),
                        attr.values.clone().expect("values is null"),
                    );
                }
                Some(_) => {
                    res.push(CollectionAttributes {
                        collection: collection.expect("Collection should be present"),
                        attributes: attributes.clone(),
                    });
                    collection = Some(col);
                    attributes.clear();
                    attributes.insert(
                        attr.trait_type.clone(),
                        attr.values.clone().expect("values is null"),
                    );
                }
            }
        }
        res.push(CollectionAttributes {
            collection: collection.expect("Collection should be present"),
            attributes: attributes.clone(),
        });
        res
    }
}

impl NFT {
    pub fn from_db(nft: crate::db::NftDetails) -> Self {
        let parsed = nft.parse_meta();

        let floor_price = match (nft.floor_price, nft.floor_price_usd, nft.floor_price_token) {
            (Some(floor_price), Some(floor_price_usd), Some(floor_price_token)) => Some(Price {
                token: floor_price_token,
                price: floor_price.to_string(),
                usd_price: Some(floor_price_usd.to_string()),
            }),
            _ => None,
        };
        NFT {
            contract: Contract {
                address: nft.address.expect("null nft address"),
                name: nft.name,
                description: nft.description,
                owner: nft.owner.map(Address::from),
                verified: None,
            },
            collection: nft.collection.unwrap_or_default(),
            manager: nft.manager.as_ref().map(Address::from),
            image: parsed.image,
            mimetype: parsed.mimetype,
            full_image: parsed.full_image,
            full_image_mimetype: parsed.full_image_mimetype,
            typ: parsed.typ,
            auction: nft.auction,
            forsale: nft.forsale,
            best_offer: nft.best_offer.clone(),
            deal_price_usd: nft.deal_price_usd.map(|it| it.to_string()),
            floor_price,
            nft_id: nft.nft_id,
            royalty: parsed.royalty,
        }
    }
}

impl Collection {
    pub fn from_db(db: crate::db::NftCollection) -> Self {
        Collection {
            contract: Contract {
                address: db.address,
                name: Some(db.name.unwrap_or_default()),
                description: Some(db.description.unwrap_or_default()),
                owner: Some(db.owner),
                verified: Some(db.verified),
            },
            verified: Some(db.verified),
            created_at: db.created.timestamp() as usize,
            logo: db.logo,
            wallpaper: db.wallpaper,
            owners_count: db.owners_count.unwrap_or_default() as usize,
            nft_count: db.nft_count as usize,
            total_price: db.total_price.map(|x| x.to_string()),
            lowest_price: None,
            first_mint: db.first_mint.timestamp(),
            social: serde_json::from_value(db.social.unwrap_or_default()).unwrap_or_default(),
            royalty: serde_json::from_value(db.royalty.unwrap_or_default()).unwrap_or_default(),
        }
    }
}

impl CollectionDetails {
    pub fn from_db(db: crate::db::NftCollectionDetails) -> anyhow::Result<Self> {
        let attributes: Vec<Attribute> = match db.attributes {
            None => vec![],
            Some(attributes) => serde_json::from_value(attributes)?,
        };

        Ok(CollectionDetails {
            collection: Collection {
                contract: Contract {
                    address: db.address.unwrap_or_default(),
                    name: Some(db.name.unwrap_or_default()),
                    description: Some(db.description.unwrap_or_default()),
                    owner: Some(db.owner.unwrap_or_default()),
                    verified: db.verified,
                },
                verified: db.verified,
                created_at: db.created.unwrap_or_default().timestamp() as usize,
                logo: db.logo,
                wallpaper: db.wallpaper,
                owners_count: db.owners_count.unwrap_or_default() as usize,
                nft_count: db.nft_count.unwrap_or_default() as usize,
                total_price: db.total_price.map(|x| x.to_string()),
                lowest_price: None,
                first_mint: db.first_mint.expect("NFT without collection").timestamp(),
                social: serde_json::from_value(db.social.unwrap_or_default())?,
                royalty: serde_json::from_value(db.royalty.unwrap_or_default())?,
            },
            floor_price_usd: db.floor_price_usd.map(|x| x.to_string()),
            total_volume_usd: db.total_volume_usd.map(|x| x.to_string()),
            attributes,
            previews: serde_json::from_value(db.previews).unwrap_or_default(),
        })
    }
}

impl Auction {
    pub fn from_db(db: &crate::db::NftAuction, tokens: &TokenDict) -> Self {
        let fee = match (db.fee_numerator, db.fee_denominator) {
            (Some(numerator), Some(denominator)) => Fee {
                numerator,
                denominator,
            },
            _ => Fee {
                numerator: 2,
                denominator: 100,
            },
        };
        let token = db.price_token.clone().unwrap_or_default();
        Auction {
            address: db.address.clone().unwrap_or_default(),
            status: db.status.clone().unwrap_or_default(),
            nft: db.nft.clone().unwrap_or_default(),
            bid_token: token.clone(),
            wallet_for_bids: db.wallet_for_bids.clone(),
            start_bid: db
                .start_price
                .clone()
                .map(|x| tokens.format_value(&token, &x)),
            start_usd_bid: db.start_usd_price.as_ref().map(|x| x.to_string()),
            max_bid: db.max_bid.clone().map(|x| tokens.format_value(&token, &x)),
            min_bid: db.min_bid.clone().map(|x| tokens.format_value(&token, &x)),
            max_usd_bid: db.max_usd_bid.as_ref().map(|x| x.to_string()),
            min_usd_bid: db.min_usd_bid.as_ref().map(|x| x.to_string()),
            start_time: db.created_at.map(|x| x.timestamp()),
            finish_time: db.finished_at.map(|x| x.timestamp()),
            last_bid_from: db.last_bid_from.clone(),
            last_bid_ts: db.last_bid_ts.map(|x| x.timestamp()),
            last_bid_value: db.last_bid_value.as_ref().map(|x| x.to_string()),
            last_bid_usd_value: db.last_bid_usd_value.as_ref().map(|x| x.to_string()),
            fee,
        }
    }
}

impl AuctionBid {
    pub fn from_db(
        bid: &crate::db::NftAuctionBid,
        auction: &crate::db::NftAuction,
        tokens: &TokenDict,
    ) -> Self {
        let token = auction.price_token.clone().unwrap_or_default();
        AuctionBid {
            from: bid.buyer.clone(),
            nft: auction.nft.clone().unwrap_or_default(),
            auction: bid.auction.clone(),
            price: tokens.format_value(&token, &bid.price),
            usd_price: bid.usd_price.as_ref().map(|x| x.to_string()),
            created_at: bid.created_at.timestamp(),
            active: bid.active,
        }
    }

    pub fn from_extended(bid: &crate::db::NftAuctionBidExt, tokens: &TokenDict) -> Self {
        let token = bid.price_token.clone().unwrap_or_default();
        AuctionBid {
            from: bid.buyer.clone(),
            nft: bid.nft.clone().unwrap_or_default(),
            auction: bid.auction.clone(),
            price: tokens.format_value(&token, &bid.price),
            usd_price: bid.usd_price.as_ref().map(|x| x.to_string()),
            created_at: bid.created_at.timestamp(),
            active: bid.active.unwrap_or_default(),
        }
    }
}

impl DirectSell {
    pub fn from_db(val: &crate::db::NftDirectSell, tokens: &TokenDict) -> Self {
        let fee = match (val.fee_numerator, val.fee_denominator) {
            (Some(numerator), Some(denominator)) => Fee {
                numerator,
                denominator,
            },
            _ => Fee {
                numerator: 2,
                denominator: 100,
            },
        };
        DirectSell {
            address: val.address.clone(),
            nft: val.nft.clone(),
            status: val.state.clone(),
            seller: val.seller.clone(),
            price: Price {
                token: val.price_token.clone(),
                price: tokens.format_value(&val.price_token, &val.price),
                usd_price: val.usd_price.as_ref().map(|x| x.to_string()),
            },
            created: val.created.timestamp(),
            finished: val.finished_at.map(|x| x.timestamp()),
            expired: val.expired_at.map(|x| x.timestamp()),
            fee,
        }
    }
}

impl DirectBuy {
    pub fn from_db(val: &crate::db::NftDirectBuy, tokens: &TokenDict) -> Self {
        let fee = match (val.fee_numerator, val.fee_denominator) {
            (Some(numerator), Some(denominator)) => Fee {
                numerator,
                denominator,
            },
            _ => Fee {
                numerator: 2,
                denominator: 100,
            },
        };
        DirectBuy {
            address: val.address.clone(),
            nft: val.nft.clone(),
            buyer: val.buyer.clone(),
            status: val.state.clone(),
            price: Price {
                token: val.price_token.clone(),
                price: tokens.format_value(&val.price_token, &val.price),
                usd_price: val.usd_price.as_ref().map(|x| x.to_string()),
            },
            created: val.created.timestamp(),
            finished: val.finished_at.map(|x| x.timestamp()),
            expired: val.expired_at.map(|x| x.timestamp()),
            fee,
        }
    }
}

impl NFTPrice {
    pub fn from_db(val: crate::db::NftPrice) -> Self {
        let usd_price = val.usd_price.to_string();
        let ts = val.ts.timestamp();
        NFTPrice { usd_price, ts }
    }
}

impl SearchResult {
    pub fn from_db(val: crate::db::SearchResult) -> Self {
        Self {
            address: val.address,
            image: val.image,
            object_type: val.object_type,
            nft_name: val.nft_name,
            collection_name: val.collection_name,
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEvents {
    pub data: Vec<NftEvent>,
    pub total_rows: i64,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEvent {
    id: i64,
    event_type: NftEventType,
    name: Option<String>,
    description: Option<String>,
    datetime: i64,
    address: String,
    preview_url: Option<String>,
    direct_sell: Option<NftEventDirectSell>,
    direct_buy: Option<NftEventDirectBuy>,
    auction: Option<NftEventAuction>,
    mint: Option<NftEventMint>,
    transfer: Option<NftEventTransfer>,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEventDirectSell {
    creator: String,
    start_time: u64,
    end_time: Option<u64>,
    duration_time: Option<i64>,
    price: String,
    usd_price: Option<String>,
    status: i64,
    payment_token: String,
    new_owner: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEventDirectBuy {
    creator: String,
    start_time: i64,
    end_time: i64,
    duration_time: Option<i64>,
    price: String,
    usd_price: Option<String>,
    status: i64,
    spent_token: String,
    old_owner: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEventAuction {
    auction_active: Option<AuctionActive>,
    auction_complete: Option<AuctionComplete>,
    auction_canceled: Option<AuctionCanceled>,
    auction_bid_placed: Option<AuctionBidPlaced>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuctionActive {
    nft_owner: String,
    auction_start_time: i64,
    auction_end_time: i64,
    auction_duration: i64,
    state: i64,
    payment_token: String,
    price: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuctionComplete {
    nft_owner: String,
    auction_start_time: i64,
    auction_end_time: i64,
    auction_duration: i64,
    state: i64,
    payment_token: String,
    price: String,
    usd_price: Option<String>,
    max_bid_value: String,
    max_bid_address: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuctionCanceled {
    nft_owner: String,
    auction_start_time: i64,
    auction_end_time: i64,
    auction_duration: i64,
    state: i64,
    payment_token: String,
    price: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuctionBidPlaced {
    bid_sender: String,
    payment_token: String,
    bid_value: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEventMint {
    owner: String,
    creator: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NftEventTransfer {
    from: String,
    to: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MetricsSummaryBase {
    total_rows_count: i32,
    data: Vec<MetricsSummary>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MetricsSummary {
    pub collection: String,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub floor_price: Option<String>,
    pub total_volume_usd_now: String,
    pub total_volume_usd_previous: String,
    pub owners_count: i32,
    pub nfts_count: i32,
}

impl From<Vec<MetricsSummaryRecord>> for MetricsSummaryBase {
    fn from(values: Vec<MetricsSummaryRecord>) -> Self {
        let total_rows_count = match values.first() {
            None => 0,
            Some(first_value) => first_value.total_rows_count,
        };

        Self {
            total_rows_count,
            data: values.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl From<MetricsSummaryRecord> for MetricsSummary {
    fn from(value: MetricsSummaryRecord) -> Self {
        Self {
            collection: value.collection,
            name: value.name,
            logo: value.logo,
            floor_price: value.floor_price.map(|t| t.to_string()),
            total_volume_usd_now: value.total_volume_usd_now.to_string(),
            total_volume_usd_previous: value.total_volume_usd_previous.to_string(),
            owners_count: value.owners_count,
            nfts_count: value.nfts_count,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OwnerFee {
    pub fee: Fee,
    pub nft_id: Option<String>,
    pub collection: Option<String>,
}

impl From<OwnerFeeRecord> for OwnerFee {
    fn from(value: OwnerFeeRecord) -> Self {
        Self {
            collection: value.collection,
            nft_id: value.nft,
            fee: Fee {
                numerator: value.fee_numerator,
                denominator: value.fee_denominator,
            },
        }
    }
}

#[derive(Debug)]
pub struct LoginData {
    pub public_key: String,
    pub address: String,
    pub wallet_type: String,
    pub timestamp: u64,
    pub signature: String,
    pub with_signature_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
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

impl From<UserRecord> for UserDto {
    fn from(value: UserRecord) -> Self {
        Self {
            address: value.address,
            logo_nft: value.logo_nft,
            username: value.username,
            bio: value.bio,
            twitter: value.twitter,
            instagram: value.instagram,
            facebook: value.facebook,
            link: value.link,
            email: value.email,
            avatar_url: value.avatar_url,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, ToSchema, Hash)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct NftsPriceRange {
    #[schema(value_type = String, format = "BigDecimal")]
    pub from: BigDecimal,
    #[schema(value_type = String, format = "BigDecimal")]
    pub to: BigDecimal,
}

impl From<NftsPriceRangeRecord> for NftsPriceRange {
    fn from(value: NftsPriceRangeRecord) -> Self {
        Self {
            from: value.from,
            to: value.to,
        }
    }
}
