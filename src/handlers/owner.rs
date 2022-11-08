use std::convert::Infallible;
use warp::http::StatusCode;
use crate::{db::{Address, Queries, DirectBuyState, DirectSellState}, model::{AuctionBid, DirectBuy, DirectSell, VecWith}};
use warp::Filter;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct OwnerParam {
    pub owner: Address,
}

/// POST /owner/bids-out
pub fn get_owner_bids_out(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("owner" / "bids-out")
        .and(warp::post())
        .and(warp::body::json::<OwnerBidsOutQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_bids_out_handler)
}

pub async fn get_owner_bids_out_handler(query: OwnerBidsOutQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let owner = query.owner;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let count = match db.list_owner_auction_bids_out_count(&owner, collections, &query.lastbid).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_owner_auction_bids_out(&owner, collections, &query.lastbid, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<AuctionBid> = list.iter().map(|x| AuctionBid::from_extended(x, &db.tokens)).collect();
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
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OwnerBidsOutQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub lastbid: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// GET /owner/bids-in
pub fn get_owner_bids_in(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("owner" / "bids-in")
        .and(warp::post())
        .and(warp::body::json::<OwnerBidsInQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_bids_in_handler)
}

pub async fn get_owner_bids_in_handler(query: OwnerBidsInQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let owner = query.owner;
    let active = &query.active;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let count = match db.list_owner_auction_bids_in_count(&owner, collections, active).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_owner_auction_bids_in(&owner, collections, active, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<AuctionBid> = list.iter().map(|x| AuctionBid::from_extended(x, &db.tokens)).collect();
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
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OwnerBidsInQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// POST /owner/direct/buy
pub fn get_owner_direct_buy(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "buy")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectBuyQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_buy_handler)
}

pub async fn get_owner_direct_buy_handler(query: OwnerDirectBuyQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_ref().map(|x| x.as_slice()).unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let count = match db.list_owner_direct_buy_count(&owner, collections, status).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_owner_direct_buy(&owner, collections, status, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<DirectBuy> = list.iter().map(|x| DirectBuy::from_db(x, &db.tokens)).collect();

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
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

/// POST /owner/direct/buy-in
pub fn get_owner_direct_buy_in(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "buy-in")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectBuyQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_buy_in_handler)
}

pub async fn get_owner_direct_buy_in_handler(query: OwnerDirectBuyQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_ref().map(|x| x.as_slice()).unwrap_or_default();
    let owner = query.owner;
    let status = query.status.as_ref().map(|x| x.as_slice()).unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let count = match db.list_owner_direct_buy_in_count(&owner, collections, status).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_owner_direct_buy_in(&owner, collections, status, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<DirectBuy> = list.iter().map(|x| DirectBuy::from_db(x, &db.tokens)).collect();
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
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}

/// POST /owner/direct/sell
pub fn get_owner_direct_sell(
    db: Queries,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "sell")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectSellQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_sell_handler)
}

pub async fn get_owner_direct_sell_handler(query: OwnerDirectSellQuery, db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_ref().map(|x| x.as_slice()).unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let count = match db.list_owner_direct_sell_count(&owner, collections, status).await {
        Err(e) => return Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(cnt) => cnt,
    };
    match db.list_owner_direct_sell(&owner, collections, status, limit, offset).await {
        Err(e) => Ok(Box::from(warp::reply::with_status(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))),
        Ok(list) => {
            let ret: Vec<DirectSell> = list.iter().map(|x| DirectSell::from_db(x, &db.tokens)).collect();
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
            Ok(Box::from(warp::reply::with_status(warp::reply::json(&ret), StatusCode::OK)))
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OwnerDirectSellQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub status: Option<Vec<DirectSellState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OwnerDirectBuyQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub status: Option<Vec<DirectBuyState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}