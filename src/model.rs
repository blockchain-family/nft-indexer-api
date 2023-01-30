use crate::db::{MetricsSummaryRecord, NftEventType};
use crate::{
    db::{Address, AuctionStatus, DirectBuyState, DirectSellState, EventCategory, EventType},
    token::TokenDict,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct VecWithTotal<T> {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VecWith<T> {
    #[serde(rename = "totalCount")]
    pub count: i64,
    pub items: Vec<T>,
    pub nft: Option<HashMap<Address, NFT>>,
    pub collection: Option<HashMap<Address, Collection>>,
    pub auction: Option<HashMap<Address, Auction>>,
    #[serde(rename = "directBuy")]
    pub direct_buy: Option<HashMap<Address, DirectBuy>>,
    #[serde(rename = "directSell")]
    pub direct_sell: Option<HashMap<Address, DirectSell>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Contract {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner: Option<Address>,
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Price {
    #[serde(rename = "priceToken")]
    pub token: Address,

    pub price: String,

    #[serde(rename = "usdPrice")]
    pub usd_price: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NFTPrice {
    #[serde(rename = "usdPrice")]
    pub usd_price: String,
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NFT {
    #[serde(flatten)]
    pub contract: Contract,

    pub collection: Address,
    pub image: Option<String>,
    pub mimetype: Option<String>,
    #[serde(rename = "type")]
    pub typ: Option<String>,
    pub attributes: Option<serde_json::Value>,

    // #[serde(rename = "currentPrice")]
    // pub current_price: Option<Price>,
    // #[serde(rename = "lastPrice")]
    // pub last_price: Option<Price>,
    pub auction: Option<Address>,
    pub forsale: Option<Address>,
    #[serde(rename = "bestOffer")]
    pub best_offer: Option<Address>,
    pub manager: Option<Address>,
    pub deal_price_usd: Option<String>,
    pub floor_price: Option<Price>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Collection {
    #[serde(flatten)]
    pub contract: Contract,

    pub verified: Option<bool>,

    #[serde(rename = "createdAt")]
    pub created_at: usize,

    pub wallpaper: Option<String>,
    pub logo: Option<String>,

    #[serde(rename = "ownersCount")]
    pub owners_count: usize,

    #[serde(rename = "nftCount")]
    pub nft_count: usize,

    #[serde(rename = "lowestPrice")]
    pub lowest_price: Option<String>,

    #[serde(rename = "totalPrice")]
    pub total_price: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollectionDetailsPreviewMeta {
    pub source: Option<String>,
    pub mimetype: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionDetails {
    #[serde(flatten)]
    pub collection: Collection,

    #[serde(rename = "floorPriceUsd")]
    pub floor_price_usd: Option<String>,

    #[serde(rename = "totalVolumeUsd")]
    pub total_volume_usd: Option<String>,
    pub attributes: Option<Value>,
    pub previews: Vec<CollectionDetailsPreviewMeta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionSimple {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub verified: bool,
    pub logo: Option<String>,
    #[serde(rename = "nftCount")]
    pub nft_count: i64,
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
    pub args: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Auction {
    pub address: Address,
    pub nft: Address,
    pub status: AuctionStatus,
    #[serde(rename = "bidToken")]
    pub bid_token: Address,
    #[serde(rename = "walletForBids")]
    pub wallet_for_bids: Option<Address>,
    #[serde(rename = "startBid")]
    pub start_bid: Option<String>,
    #[serde(rename = "startUsdBid")]
    pub start_usd_bid: Option<String>,
    #[serde(rename = "minBid")]
    pub min_bid: Option<String>,
    #[serde(rename = "minUsdBid")]
    pub min_usd_bid: Option<String>,
    #[serde(rename = "maxBid")]
    pub max_bid: Option<String>,
    #[serde(rename = "maxUsdBid")]
    pub max_usd_bid: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<i64>,
    #[serde(rename = "finishTime")]
    pub finish_time: Option<i64>,
    #[serde(rename = "lastBidFrom")]
    pub last_bid_from: Option<Address>,
    #[serde(rename = "lastBidTime")]
    pub last_bid_ts: Option<i64>,
    #[serde(rename = "lastBidValue")]
    pub last_bid_value: Option<String>,
    #[serde(rename = "lastBidUsdValue")]
    pub last_bid_usd_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuctionBid {
    pub from: Address,
    pub auction: Address,
    pub nft: Address,
    pub price: String,
    #[serde(rename = "usdPrice")]
    pub usd_price: Option<String>,
    pub active: bool,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionAttributes {
    pub collection: Address,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub address: Address,
    pub image: Option<String>,
    #[serde(rename = "contractType")]
    pub contract_type: EventCategory,
}

impl CollectionAttributes {
    pub fn from_db(defs: &[crate::db::TraitDef]) -> Vec<Self> {
        let mut res = Vec::with_capacity(16);
        let mut collection = None;
        let mut attributes = HashMap::new();
        for attr in defs.iter() {
            let col = attr.collection.clone().unwrap();
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
                        collection: collection.unwrap(),
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
            collection: collection.unwrap(),
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
            typ: parsed.typ,
            attributes: parsed.attributes,
            auction: nft.auction,
            forsale: nft.forsale,
            best_offer: nft.best_offer.clone(),
            deal_price_usd: nft.deal_price_usd.map(|it| it.to_string()),
            floor_price, // current_price: None,
                         // last_price: None,
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
        }
    }
}

impl CollectionDetails {
    pub fn from_db(db: crate::db::NftCollectionDetails) -> Self {
        CollectionDetails {
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
            },
            floor_price_usd: db.floor_price_usd.map(|x| x.to_string()),
            total_volume_usd: db.total_volume_usd.map(|x| x.to_string()),
            attributes: db.attributes,
            previews: serde_json::from_value(db.previews).unwrap_or_default(),
        }
    }
}

impl Auction {
    pub fn from_db(db: &crate::db::NftAuction, tokens: &TokenDict) -> Self {
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
        }
    }
}

impl DirectBuy {
    pub fn from_db(val: &crate::db::NftDirectBuy, tokens: &TokenDict) -> Self {
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
        }
    }
}

impl NFTPrice {
    pub fn from_db(val: &crate::db::NftPrice) -> Self {
        let usd_price = val
            .usd_price
            .as_ref()
            .expect("null usd_price in price history")
            .round(0)
            .to_string();
        let ts = val.ts.expect("null ts in price history").timestamp();
        NFTPrice { usd_price, ts }
    }
}

impl SearchResult {
    pub fn from_db(val: &crate::db::SearchResult) -> Self {
        Self {
            address: val.address.clone(),
            image: val.image.clone(),
            contract_type: val.typ.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftEvents {
    data: Vec<NftEvent>,
    total_rows: i64,
}

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NftEventAuction {
    auction_active: Option<AuctionActive>,
    auction_complete: Option<AuctionComplete>,
    auction_canceled: Option<AuctionCanceled>,
    auction_bid_placed: Option<AuctionBidPlaced>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuctionActive {
    nft_owner: String,
    auction_start_time: i64,
    auction_end_time: i64,
    auction_duration: i64,
    state: i64,
    payment_token: String,
    price: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuctionComplete {
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuctionCanceled {
    nft_owner: String,
    auction_start_time: i64,
    auction_end_time: i64,
    auction_duration: i64,
    state: i64,
    payment_token: String,
    price: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuctionBidPlaced {
    bid_sender: String,
    payment_token: String,
    bid_value: String,
    usd_price: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NftEventMint {
    owner: String,
    creator: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NftEventTransfer {
    from: String,
    to: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsSummaryBase {
    total_rows_count: i32,
    data: Vec<MetricsSummary>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetricsSummary {
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
