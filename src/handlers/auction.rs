use serde::{Serialize, Deserialize};
use std::{convert::Infallible, collections::HashMap};
use warp::{http::StatusCode, Filter};
use crate::db::{Address, Queries};
use crate::model::{NFT, Collection, Auction, AuctionBid, VecWith};


/// POST /auctions
pub fn get_auctions(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auctions")
        .and(warp::post())
        .and(warp::body::json::<AuctionsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auctions_handler)
}

pub async fn get_auctions_handler(params: AuctionsQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owners: &[String] = params.owners.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let collections = params.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let tokens = params.tokens.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let sort = params.sort.clone().unwrap_or(AuctionsSortOrder::StartDate);
    let count = match db.list_nft_auctions_count(owners, collections, tokens).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_nft_auctions(owners, collections, tokens, &sort, params.limit.unwrap_or(100), params.offset.unwrap_or_default()).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<Auction> = list.iter().map(|col| Auction::from_db(col, &db.tokens)).collect();
            let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
            let (nft, collection) = match super::collect_nft_and_collection(&db, &nft_ids).await {
                Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
                Ok(m) => m,
            };
            let ret = VecWith { 
                count, items: ret,
                nft: Some(nft),
                collection: Some(collection),
                auction: None,
                direct_buy: None,
                direct_sell: None,
            };
            Ok(Box::from(
                warp::reply::with_status(
                    warp::reply::json(&ret), 
                    StatusCode::OK)
            ))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuctionsQuery {
    pub owners: Option<Vec<Address>>,
    pub collections: Option<Vec<Address>>,
    pub tokens: Option<Vec<Address>>,
    pub sort: Option<AuctionsSortOrder>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuctionBidsQuery {
    pub auction: Address,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AuctionsSortOrder {
    #[serde(rename = "start-date")]
    StartDate,
    #[serde(rename = "bids-count")]
    BidsCount,
    #[serde(rename = "average")]
    Average,
    #[serde(rename = "average-in-hour")]
    AverageInHour,
    #[serde(rename = "average-in-day")]
    AverageInDay,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetAuctionResult {
    pub auction: Auction,
    pub bid: Option<AuctionBid>,
    pub nft: HashMap<Address, NFT>,
    pub collection: HashMap<Address, Collection>,
}

/// POST /auction
pub fn get_auction(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auction")
        .and(warp::post())
        .and(warp::body::json::<AuctionBidsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auction_handler)
}

pub async fn get_auction_handler(params: AuctionBidsQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let auction = match db.get_nft_auction(&params.auction).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(None) => return Ok(Box::from(warp::reply::with_status("auction not found".to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(Some(a)) => a,
    };
    let nft_ids = vec![auction.nft.clone().unwrap_or_default()];
    let (nft, collection) = match super::collect_nft_and_collection(&db, &nft_ids).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(m) => m,
    };

    let bid = match db.get_nft_auction_last_bid(&params.auction).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(None) => None,
        Ok(Some(b)) => Some(AuctionBid::from_db(&b, &auction, &db.tokens)),
    };
    let auction = Auction::from_db(&auction, &db.tokens);
    let ret = GetAuctionResult { auction, nft, collection, bid };
    Ok(Box::from(
        warp::reply::with_status(
            warp::reply::json(&ret), 
            StatusCode::OK)
    ))
}

/// POST /auction/{address}/bids
pub fn get_auction_bids(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("auction" / "bids")
        .and(warp::post())
        .and(warp::body::json::<AuctionBidsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auction_bids_handler)
}

pub async fn get_auction_bids_handler(params: AuctionBidsQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let auc = match db.get_nft_auction(&params.auction).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(None) => return Ok(Box::from(warp::reply::with_status("auction not found".to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(Some(a)) => a,
    };
    let count = match db.list_nft_auction_bids_count(&params.auction).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_nft_auction_bids(&params.auction, params.limit.unwrap_or(100), params.offset.unwrap_or_default()).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(bids) => {
            let ret: Vec<AuctionBid> = bids
                .iter()
                .map(|b| AuctionBid::from_db(b, &auc, &db.tokens))
                .collect();

            let auction_ids: Vec<String> = ret.iter().map(|x| x.auction.clone()).collect();
            let (nft, collection, auctions) = match super::collect_auctions_nfts_collections(&db, &auction_ids).await {
                Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
                Ok(m) => m,
            };

            let ret = VecWith { 
                count, items: ret,
                nft: Some(nft),
                collection: Some(collection),
                auction: Some(auctions),
                direct_buy: None,
                direct_sell: None,
            };
            Ok(Box::from(
                warp::reply::with_status(
                    warp::reply::json(&ret), 
                    StatusCode::OK)
            ))
        },
    }
}

pub async fn collect_auctions(db: &Queries, ids: &Vec<String>) -> anyhow::Result<HashMap<String, Auction>> {
    let dblist = db.collect_auctions(ids).await?;
    let list = dblist
        .iter()
        .map(|col| Auction::from_db(col, &db.tokens));
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.address.clone(), item.clone());
    }
    Ok(map)
}

pub async fn collect_auctions_nfts_collections(db: &Queries,
    auction_ids: &Vec<String>,
) -> anyhow::Result<(HashMap<String, NFT>, HashMap<String, Collection>, HashMap<String, Auction>)> {
    let auctions = collect_auctions(db, auction_ids).await?;
    let nft_ids = auctions.values().map(|x| x.nft.clone()).collect();
    let (nft, collection) = super::collect_nft_and_collection(db, &nft_ids).await?;
    Ok((nft, collection, auctions))
}
