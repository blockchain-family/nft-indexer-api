use super::HttpState;
use crate::db::RootType;
use crate::handlers::auction::collect_auctions_nfts_collections;
use crate::handlers::nft::collect_nft_and_collection;
use crate::model::OwnerFee;
use crate::schema::VecWithAuctionBids;
use crate::schema::VecWithDirectBuy;
use crate::schema::VecWithDirectSell;
use crate::{
    catch_error_500,
    db::{Address, DirectBuyState, DirectSellState},
    model::{AuctionBid, DirectBuy, DirectSell, VecWith},
    response,
};
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::IntoParams;
use utoipa::ToSchema;

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
pub async fn get_owner_bids_out(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<OwnerBidsOutQuery>,
) -> impl IntoResponse {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        s.db.list_owner_auction_bids_out(&owner, collections, &query.lastbid, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<AuctionBid> = list
        .iter()
        .map(|x| AuctionBid::from_extended(x, &s.db.tokens))
        .collect();
    let auction_ids: Vec<String> = ret.iter().map(|x| x.auction.clone()).collect();
    let (nft, collection, auctions) =
        catch_error_500!(collect_auctions_nfts_collections(&s.db, &auction_ids).await);

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
pub async fn get_owner_bids_in(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<OwnerBidsInQuery>,
) -> impl IntoResponse {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let active = &query.active;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        s.db.list_owner_auction_bids_in(&owner, collections, active, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<AuctionBid> = list
        .iter()
        .map(|x| AuctionBid::from_extended(x, &s.db.tokens))
        .collect();
    let auction_ids: Vec<String> = ret.iter().map(|x| x.auction.clone()).collect();
    let (nft, collection, auctions) =
        catch_error_500!(collect_auctions_nfts_collections(&s.db, &auction_ids).await);

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
pub async fn get_owner_direct_buy(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<OwnerDirectBuyQuery>,
) -> impl IntoResponse {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        s.db.list_owner_direct_buy(&owner, collections, status, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<DirectBuy> = list
        .iter()
        .map(|x| DirectBuy::from_db(x, &s.db.tokens))
        .collect();

    let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&s.db, &nft_ids).await);

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
pub async fn get_owner_direct_buy_in(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<OwnerDirectBuyQuery>,
) -> impl IntoResponse {
    let collections = query.collections.as_deref().unwrap_or_default();
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        s.db.list_owner_direct_buy_in(&owner, collections, status, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<DirectBuy> = list
        .iter()
        .map(|x| DirectBuy::from_db(x, &s.db.tokens))
        .collect();
    let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&s.db, &nft_ids).await);

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
pub async fn get_owner_direct_sell(
    State(s): State<Arc<HttpState>>,
    Json(query): Json<OwnerDirectSellQuery>,
) -> impl IntoResponse {
    let collections = query.collections.as_deref().unwrap_or(&[]);
    let owner = query.owner;
    let status = query.status.as_deref().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or_default();
    let list = catch_error_500!(
        s.db.list_owner_direct_sell(&owner, collections, status, limit, offset)
            .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<DirectSell> = list
        .iter()
        .map(|x| DirectSell::from_db(x, &s.db.tokens))
        .collect();
    let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();
    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&s.db, &nft_ids).await);

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
pub async fn get_fee(
    State(s): State<Arc<HttpState>>,
    Query(query): Query<OwnerFeeQuery>,
) -> impl IntoResponse {
    let fee = catch_error_500!(s.db.get_owner_fee(&query.owner, &query.root_code).await);

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
