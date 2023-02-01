use crate::db::NftDetails;
use crate::handlers::OrderDirection;
use crate::model::{DirectBuy, NFTPrice, VecWith, NFT};
use crate::{
    db::{Address, DirectBuyState, Queries},
    model::{Auction, Collection, DirectSell},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Display;
use warp::http::StatusCode;
use warp::Filter;

use super::collect_collections;

/// POST /nft/details
pub fn get_nft(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nft" / "details")
        .and(warp::post())
        .and(warp::body::json::<NFTParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nft_handler)
}

pub async fn get_nft_handler(
    param: NFTParam,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    match db.get_nft_details(&param.nft).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
        Ok(None) => Ok(Box::from(warp::reply::with_status(
            String::default(),
            StatusCode::BAD_REQUEST,
        ))),
        Ok(Some(mut nft)) => {
            let collections_ids = match &nft.collection {
                Some(c) => vec![c.clone()],
                None => vec![],
            };
            let collection = match super::collect_collections(&db, &collections_ids).await {
                Err(e) => {
                    return Ok(Box::from(warp::reply::with_status(
                        e.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )))
                }
                Ok(m) => m,
            };
            let mut auction = HashMap::default();
            if nft.auction.is_none() {
                match db
                    .get_nft_auction_by_nft(&nft.address.clone().unwrap_or_default())
                    .await
                {
                    Err(e) => {
                        return Ok(Box::from(warp::reply::with_status(
                            e.to_string(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )))
                    }
                    Ok(Some(a)) => nft.auction = a.address,
                    _ => {}
                }
            };
            if let Some(ref auction_id) = nft.auction {
                match db.get_nft_auction(auction_id).await {
                    Err(e) => {
                        return Ok(Box::from(warp::reply::with_status(
                            e.to_string(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )))
                    }
                    Ok(Some(a)) => {
                        auction.insert(auction_id.clone(), Auction::from_db(&a, &db.tokens));
                    }
                    _ => {}
                }
            };

            let mut direct_sell = HashMap::default();
            if nft.forsale.is_none() {
                match db
                    .get_nft_direct_sell(&nft.address.clone().unwrap_or_default())
                    .await
                {
                    Err(e) => {
                        return Ok(Box::from(warp::reply::with_status(
                            e.to_string(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )))
                    }
                    Ok(Some(a)) => nft.forsale = Some(a.address),
                    _ => {}
                }
            }
            if let Some(ref direct_sell_id) = nft.forsale {
                match db.get_direct_sell(direct_sell_id).await {
                    Err(e) => {
                        return Ok(Box::from(warp::reply::with_status(
                            e.to_string(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )))
                    }
                    Ok(Some(a)) => {
                        direct_sell
                            .insert(direct_sell_id.clone(), DirectSell::from_db(&a, &db.tokens));
                    }
                    _ => {}
                }
            };

            let mut direct_buy = HashMap::default();
            let nft_addr = nft.address.clone().unwrap_or_default();
            match db
                .list_nft_direct_buy(&nft_addr, &[DirectBuyState::Active], 100, 0)
                .await
            {
                Err(e) => {
                    return Ok(Box::from(warp::reply::with_status(
                        e.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )))
                }
                Ok(mut list) => {
                    for x in list.drain(..) {
                        direct_buy.insert(x.address.clone(), DirectBuy::from_db(&x, &db.tokens));
                    }
                }
            }

            let ret = GetNFTResult {
                nft: NFT::from_db(nft /*, &db.tokens*/),
                collection,
                auction,
                direct_buy,
                direct_sell,
            };
            Ok(Box::from(warp::reply::with_status(
                warp::reply::json(&ret),
                StatusCode::OK,
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetNFTResult {
    pub nft: NFT,
    pub collection: HashMap<Address, Collection>,
    pub auction: HashMap<Address, Auction>,
    #[serde(rename = "directSell")]
    pub direct_sell: HashMap<Address, DirectSell>,
    #[serde(rename = "directBuy")]
    pub direct_buy: HashMap<Address, DirectBuy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NFTParam {
    pub nft: Address,
    pub status: Option<Vec<DirectBuyState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// POST /nft/direct/buy
pub fn get_nft_direct_buy(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nft" / "direct" / "buy")
        .and(warp::post())
        .and(warp::body::json::<NFTParam>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nft_direct_buy_handler)
}

pub async fn get_nft_direct_buy_handler(
    params: NFTParam,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or_default();
    let nft = params.nft;
    let status = params.status.as_deref().unwrap_or_default();
    match db.list_nft_direct_buy(&nft, status, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
        Ok(list) => {
            let count = list.first().map(|it| it.cnt).unwrap_or_default();
            let ret: Vec<DirectBuy> = list
                .iter()
                .map(|x| DirectBuy::from_db(x, &db.tokens))
                .collect();
            let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
            let (nft, collection) = match collect_nft_and_collection(&db, &nft_ids).await {
                Err(e) => {
                    return Ok(Box::from(warp::reply::with_status(
                        e.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    )))
                }
                Ok(m) => m,
            };
            let ret = VecWith {
                count,
                items: ret,
                nft: Some(nft),
                collection: Some(collection),
                auction: None,
                direct_buy: None,
                direct_sell: None,
            };
            Ok(Box::from(warp::reply::with_status(
                warp::reply::json(&ret),
                StatusCode::OK,
            )))
        }
    }
}

/// POST /nft/price-history
pub fn get_nft_price_history(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nft" / "price-history")
        .and(warp::post())
        .and(warp::body::json::<NftPriceHistoryQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nft_price_history_handler)
}

pub async fn get_nft_price_history_handler(
    query: NftPriceHistoryQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let ret = match query.scale.unwrap_or_default() {
        PriceHistoryScale::Days => {
            db.list_nft_price_history_days(&query.nft, query.from, query.to)
                .await
        }
        PriceHistoryScale::Hours => {
            db.list_nft_price_history_hours(&query.nft, query.from, query.to)
                .await
        }
    };
    match ret {
        Err(e) => Ok(Box::from(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
        Ok(list) => {
            let ret: Vec<NFTPrice> = list.iter().map(NFTPrice::from_db).collect();
            Ok(Box::from(warp::reply::with_status(
                warp::reply::json(&ret),
                StatusCode::OK,
            )))
        }
    }
}

/// POST /nft/{address}/reload-meta
pub fn post_nft_reload_meta(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nft" / Address / "reload-meta")
        .and(warp::post())
        .and(warp::any().map(move || db.clone()))
        .and_then(post_nft_reload_meta_handler)
}

pub async fn post_nft_reload_meta_handler(
    _address: Address,
    _db: Queries,
) -> Result<impl warp::Reply, Infallible> {
    Ok(StatusCode::OK)
}

/// POST /nfts/top
pub fn get_nft_top_list(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "top")
        .and(warp::post())
        .and(warp::body::json::<NFTTopListQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nft_top_list_handler)
}
#[derive(Clone, Deserialize, Serialize)]
pub struct NFTTopListQuery {
    pub from: i64,
    pub limit: i64,
    pub offset: i64,
}

/// POST /nfts/
pub fn get_nft_list(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("nfts")
        .and(warp::post())
        .and(warp::body::json::<NFTListQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nft_list_handler)
}

pub async fn get_nft_top_list_handler(
    params: NFTTopListQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let from = NaiveDateTime::from_timestamp(params.from, 0);
    match db.nft_top_search(from, params.limit, params.offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
        Ok(list) => {
            let response = make_nfts_response(list, db).await;
            match response {
                Ok(response) => Ok(Box::from(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::OK,
                ))),
                Err(e) => Ok(Box::from(warp::reply::with_status(
                    e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))),
            }
        }
    }
}

pub async fn get_nft_list_handler(
    params: NFTListQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owners = params.owners.as_deref().unwrap_or(&[]);
    let collections = params.collections.as_deref().unwrap_or(&[]);
    let verified = Some(params.verified.unwrap_or(true));

    match db
        .nft_search(
            owners,
            collections,
            params.price_from,
            params.price_to,
            params.price_token,
            params.forsale,
            params.auction,
            verified,
            params.limit.unwrap_or(100),
            params.offset.unwrap_or_default(),
            &params.attributes.unwrap_or_default(),
            params.order,
        )
        .await
    {
        Err(e) => Ok(Box::from(warp::reply::with_status(
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))),
        Ok(list) => {
            let response = make_nfts_response(list, db).await;

            match response {
                Ok(response) => Ok(Box::from(warp::reply::with_status(
                    warp::reply::json(&response),
                    StatusCode::OK,
                ))),
                Err(e) => Ok(Box::from(warp::reply::with_status(
                    e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))),
            }
        }
    }
}

async fn make_nfts_response(list: Vec<NftDetails>, db: Queries) -> anyhow::Result<VecWith<NFT>> {
    let count = match list.first() {
        None => 0,
        Some(first) => first.total_count,
    };
    let ret: Vec<NFT> = list.iter().map(|it| NFT::from_db(it.clone())).collect();
    let collection_ids = ret.iter().map(|x| x.collection.clone()).collect();
    let collection = collect_collections(&db, &collection_ids).await?;

    let auction_ids: Vec<String> = list.iter().filter_map(|x| x.auction.clone()).collect();
    let auction = super::collect_auctions(&db, &auction_ids).await?;

    let direct_sell_ids: Vec<String> = list.iter().filter_map(|x| x.forsale.clone()).collect();
    let direct_sell = super::collect_direct_sell(&db, &direct_sell_ids).await?;

    let direct_buy_ids: Vec<String> = list.iter().filter_map(|x| x.best_offer.clone()).collect();
    let direct_buy = super::collect_direct_buy(&db, &direct_buy_ids).await?;
    Ok(VecWith {
        count,
        items: ret,
        collection: Some(collection),
        nft: None,
        auction: Some(auction),
        direct_buy: Some(direct_buy),
        direct_sell: Some(direct_sell),
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributeFilter {
    #[serde(rename = "traitType")]
    pub trait_type: String,
    #[serde(rename = "traitValues")]
    pub trait_values: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NFTListQuery {
    pub owners: Option<Vec<String>>,
    pub collections: Option<Vec<String>>,
    #[serde(rename = "priceFrom")]
    pub price_from: Option<u64>,
    #[serde(rename = "priceTo")]
    pub price_to: Option<u64>,
    #[serde(rename = "priceToken")]
    pub price_token: Option<String>,
    pub forsale: Option<bool>,
    pub auction: Option<bool>,
    pub verified: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub attributes: Option<Vec<AttributeFilter>>,
    pub order: Option<NFTListOrder>,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum NFTListOrderField {
    #[serde(rename = "floorPriceUsd")]
    FloorPriceUsd,
    #[serde(rename = "dealPriceUsd")]
    DealPriceUsd,
}

impl Display for NFTListOrderField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NFTListOrderField::FloorPriceUsd => write!(f, "floor_price_usd"),
            NFTListOrderField::DealPriceUsd => write!(f, "deal_price_usd"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NFTListOrder {
    pub field: NFTListOrderField,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PriceHistoryScale {
    #[serde(rename = "h")]
    Hours,
    #[serde(rename = "d")]
    Days,
}

impl Default for PriceHistoryScale {
    fn default() -> Self {
        Self::Days
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftPriceHistoryQuery {
    pub nft: Address,
    pub scale: Option<PriceHistoryScale>,
    pub from: Option<usize>,
    pub to: Option<usize>,
}

pub async fn collect_nfts(db: &Queries, ids: &[String]) -> anyhow::Result<HashMap<String, NFT>> {
    let dblist = db.collect_nfts(ids).await?;
    let list = dblist.into_iter().map(NFT::from_db);
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.contract.address.clone(), item.clone());
    }
    Ok(map)
}

pub async fn collect_nft_and_collection(
    db: &Queries,
    nft_ids: &Vec<String>,
) -> anyhow::Result<(HashMap<String, NFT>, HashMap<String, Collection>)> {
    let nft = collect_nfts(db, nft_ids).await?;
    let collection_ids = nft.values().map(|x| x.collection.clone()).collect();
    let collection = collect_collections(db, &collection_ids).await?;
    Ok((nft, collection))
}

pub async fn collect_direct_sell(
    db: &Queries,
    ids: &[String],
) -> anyhow::Result<HashMap<String, DirectSell>> {
    let dblist = db.collect_direct_sell(ids).await?;
    let list = dblist
        .iter()
        .map(|col| DirectSell::from_db(col, &db.tokens));
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.address.clone(), item.clone());
    }
    Ok(map)
}

pub async fn collect_direct_buy(
    db: &Queries,
    ids: &[String],
) -> anyhow::Result<HashMap<String, DirectBuy>> {
    let dblist = db.collect_direct_buy(ids).await?;
    let list = dblist.iter().map(|col| DirectBuy::from_db(col, &db.tokens));
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.address.clone(), item.clone());
    }
    Ok(map)
}
