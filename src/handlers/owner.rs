use crate::db::queries::Queries;
use crate::db::RootType;
use crate::handlers::auction::collect_auctions_nfts_collections;
use crate::handlers::nft::collect_nft_and_collection;
use crate::model::OwnerFee;
use crate::schema::VecWithAuctionBids;
use crate::schema::VecWithDirectBuy;
use crate::schema::VecWithDirectSell;
use crate::{
    api_doc_addon, catch_error_500,
    db::{Address, DirectBuyState, DirectSellState},
    model::{AuctionBid, DirectBuy, DirectSell, VecWith},
    response,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use utoipa::IntoParams;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_owner_bids_out,
        get_owner_bids_in,
        get_owner_direct_buy,
        get_owner_direct_buy_in,
        get_owner_direct_sell,
        get_fee
    ),
    components(schemas(
        OwnerBidsOutQuery,
        VecWithAuctionBids,
        OwnerDirectBuyQuery,
        OwnerDirectSellQuery,
        VecWithDirectSell,
        VecWithDirectBuy,
        OwnerBidsInQuery,
        RootType,
        OwnerFee
    )),
    tags(
        (name = "owner", description = "Owner handlers"),
    )
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Debug, Clone, Deserialize)]
pub struct OwnerParam {
    pub owner: Address,
}

#[utoipa::path(
    post,
    tag = "owner",
    path = "/owner/bids-out",
    request_body(content = OwnerBidsOutQuery, description = "Get bids out by owner"),
    responses(
        (status = 200, body = VecWithAuctionBids),
        (status = 500),
    ),
)]
pub fn get_owner_bids_out(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "bids-out")
        .and(warp::post())
        .and(warp::body::json::<OwnerBidsOutQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_bids_out_handler)
}

pub async fn get_owner_bids_out_handler(
    query: OwnerBidsOutQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_owner_auction_bids_out(&owner, collections, &query.lastbid, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<AuctionBid> = list
        .iter()
        .map(|x| AuctionBid::from_extended(x, &db.tokens))
        .collect();
    let auction_ids: Vec<String> = ret.iter().map(|x| x.auction.clone()).collect();
    let (nft, collection, auctions) =
        catch_error_500!(collect_auctions_nfts_collections(&db, &auction_ids).await);

    let ret = VecWith {
        count,
        items: ret,
        nft: Some(nft),
        collection: Some(collection),
        auction: Some(auctions),
        direct_buy: None,
        direct_sell: None,
    };
    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OwnerBidsOutQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub lastbid: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
#[utoipa::path(
    tag = "owner",
    post,
    path = "/owner/bids-in",
    request_body(content = OwnerBidsInQuery, description = "Get bids in by owner"),
    responses(
        (status = 200, body = VecWithAuctionBids),
        (status = 500),
    ),
)]
pub fn get_owner_bids_in(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "bids-in")
        .and(warp::post())
        .and(warp::body::json::<OwnerBidsInQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_bids_in_handler)
}

pub async fn get_owner_bids_in_handler(
    query: OwnerBidsInQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let active = &query.active;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_owner_auction_bids_in(&owner, collections, active, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<AuctionBid> = list
        .iter()
        .map(|x| AuctionBid::from_extended(x, &db.tokens))
        .collect();
    let auction_ids: Vec<String> = ret.iter().map(|x| x.auction.clone()).collect();
    let (nft, collection, auctions) =
        catch_error_500!(collect_auctions_nfts_collections(&db, &auction_ids).await);

    let ret = VecWith {
        count,
        items: ret,
        nft: Some(nft),
        collection: Some(collection),
        auction: Some(auctions),
        direct_buy: None,
        direct_sell: None,
    };
    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OwnerBidsInQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[utoipa::path(
    tag = "owner",
    post,
    path = "/owner/direct/buy",
    request_body(content = OwnerDirectBuyQuery, description = "Get direct buys by owner"),
    responses(
        (status = 200, body = VecWithDirectBuy),
        (status = 500),
    ),
)]
pub fn get_owner_direct_buy(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "buy")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectBuyQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_buy_handler)
}

pub async fn get_owner_direct_buy_handler(
    query: OwnerDirectBuyQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_owner_direct_buy(&owner, collections, status, limit, offset)
            .await
    );

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
    response!(&ret)
}

#[utoipa::path(
    tag = "owner",
    post,
    path = "/owner/direct/buy-in",
    request_body(content = OwnerDirectBuyQuery, description = "Get NFT direct buy in by owner"),
    responses(
        (status = 200, body = VecWithDirectBuy),
        (status = 500),
    ),
)]
pub fn get_owner_direct_buy_in(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "buy-in")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectBuyQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_buy_in_handler)
}

pub async fn get_owner_direct_buy_in_handler(
    query: OwnerDirectBuyQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_deref().unwrap_or_default();
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_owner_direct_buy_in(&owner, collections, status, limit, offset)
            .await
    );

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
    response!(&ret)
}

#[utoipa::path(
    tag = "owner",
    post,
    path = "/owner/direct/sell",
    request_body(content = OwnerDirectSellQuery, description = "Get direct sell by owner"),
    responses(
        (status = 200, body = VecWithDirectSell),
        (status = 500),
    ),
)]
pub fn get_owner_direct_sell(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "direct" / "sell")
        .and(warp::post())
        .and(warp::body::json::<OwnerDirectSellQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_owner_direct_sell_handler)
}

pub async fn get_owner_direct_sell_handler(
    query: OwnerDirectSellQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        db.list_owner_direct_sell(&owner, collections, status, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<DirectSell> = list
        .iter()
        .map(|x| DirectSell::from_db(x, &db.tokens))
        .collect();
    let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&db, &nft_ids).await);

    let ret = VecWith {
        count,
        items: ret,
        nft: Some(nft),
        collection: Some(collection),
        auction: None,
        direct_buy: None,
        direct_sell: None,
    };
    response!(&ret)
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OwnerFeeQuery {
    pub owner: Address,
    #[serde(rename = "rootCode")]
    pub root_code: RootType,
}

#[utoipa::path(
tag = "owner",
    get,
    path = "/owner/fee",
    params(OwnerFeeQuery),
    responses(
        (status = 200, body = OwnerFee),
        (status = 500),
    ),
)]
pub fn get_fee(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("owner" / "fee")
        .and(warp::get())
        .and(warp::query::<OwnerFeeQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_fee_handler)
}

pub async fn get_fee_handler(
    query: OwnerFeeQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let fee = catch_error_500!(db.get_owner_fee(&query.owner, &query.root_code).await);

    let owner_fee = OwnerFee::from(fee);
    response!(&owner_fee)
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OwnerDirectSellQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub status: Option<Vec<DirectSellState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct OwnerDirectBuyQuery {
    pub owner: Address,
    pub collections: Option<Vec<Address>>,
    pub status: Option<Vec<DirectBuyState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
