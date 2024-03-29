use crate::db::queries::Queries;
use crate::db::Address;
use crate::handlers::nft::collect_nft_and_collection;
use crate::model::{Auction, AuctionBid, Collection, VecWith, NFT};
use crate::{api_doc_addon, catch_empty, catch_error_500, response, schema};
use schema::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible};
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::{http::StatusCode, Filter};
#[derive(OpenApi)]
#[openapi(
    paths(get_auctions, get_auction, get_auction_bids),
    components(schemas(
        AuctionsQuery,
        AuctionsSortOrder,
        VecWithAuction,
        AuctionBidsQuery,
        GetAuctionResult,
        VecWithAuctionBids
    )),
    tags(
        (name = "auction", description = "Auction handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[utoipa::path(
    post,
    tag = "auction",
    path = "/auctions",
    request_body(content = AuctionsQuery, description = "Auction list"),
    responses(
        (status = 200, body = VecWithAuction),
        (status = 500),
    ),
)]
pub fn get_auctions(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("auctions")
        .and(warp::post())
        .and(warp::body::json::<AuctionsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auctions_handler)
}

pub async fn get_auctions_handler(
    params: AuctionsQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let owners: &[String] = params.owners.as_deref().unwrap_or(&[]);
    let collections = params.collections.as_deref().unwrap_or(&[]);
    let tokens = params.tokens.as_deref().unwrap_or(&[]);
    let sort = params.sort.clone().unwrap_or(AuctionsSortOrder::StartDate);
    let list = catch_error_500!(
        db.list_nft_auctions(
            owners,
            collections,
            tokens,
            &sort,
            params.limit.unwrap_or(100),
            params.offset.unwrap_or_default(),
        )
        .await
    );

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<Auction> = list
        .iter()
        .map(|col| Auction::from_db(col, &db.tokens))
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuctionsQuery {
    pub owners: Option<Vec<Address>>,
    pub collections: Option<Vec<Address>>,
    pub tokens: Option<Vec<Address>>,
    pub sort: Option<AuctionsSortOrder>,
    pub limit: Option<usize>,
    #[schema(example = 1001)]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuctionBidsQuery {
    pub auction: Address,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetAuctionResult {
    pub auction: Auction,
    pub bid: Option<AuctionBid>,
    pub nft: HashMap<Address, NFT>,
    pub collection: HashMap<Address, Collection>,
}

#[utoipa::path(
    post,
    tag = "auction",
    path = "/auction",
    request_body(content = AuctionBidsQuery, description = "Show auction"),
    responses(
        (status = 200, body = GetAuctionResult),
        (status = 500),
    ),
)]

pub fn get_auction(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("auction")
        .and(warp::post())
        .and(warp::body::json::<AuctionBidsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auction_handler)
}

pub async fn get_auction_handler(
    params: AuctionBidsQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let auction = catch_error_500!(db.get_nft_auction(&params.auction).await);
    let auction = catch_empty!(auction, "auction not found");

    let nft_ids = vec![auction.nft.clone().unwrap_or_default()];
    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&db, &nft_ids).await);

    let bid = catch_error_500!(db.get_nft_auction_last_bid(&params.auction).await);
    let bid = bid.map(|b| AuctionBid::from_db(&b, &auction, &db.tokens));

    let auction = Auction::from_db(&auction, &db.tokens);
    let ret = GetAuctionResult {
        auction,
        nft,
        collection,
        bid,
    };

    response!(&ret)
}

#[utoipa::path(
    post,
    tag = "auction",
    path = "/auction/bids",
    request_body(content = AuctionBidsQuery, description = "Auction bids"),
    responses(
        (status = 200, body = VecWithAuctionBids),
        (status = 500),
    ),
)]
pub fn get_auction_bids(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("auction" / "bids")
        .and(warp::post())
        .and(warp::body::json::<AuctionBidsQuery>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_auction_bids_handler)
}

pub async fn get_auction_bids_handler(
    params: AuctionBidsQuery,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let auc = catch_error_500!(db.get_nft_auction(&params.auction).await);
    let auc = catch_empty!(auc, "auction not found");

    let bids = catch_error_500!(
        db.list_nft_auction_bids(
            &params.auction,
            params.limit.unwrap_or(100),
            params.offset.unwrap_or_default(),
        )
        .await
    );

    let count = bids.first().map(|it| it.cnt).unwrap_or_default();

    let ret: Vec<AuctionBid> = bids
        .iter()
        .map(|b| AuctionBid::from_db(b, &auc, &db.tokens))
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

pub async fn collect_auctions(
    db: &Queries,
    ids: &[String],
) -> anyhow::Result<HashMap<String, Auction>> {
    let dblist = db.collect_auctions(ids).await?;
    let list = dblist.iter().map(|col| Auction::from_db(col, &db.tokens));
    let mut map = HashMap::new();
    for item in list {
        map.insert(item.address.clone(), item.clone());
    }
    Ok(map)
}

pub async fn collect_auctions_nfts_collections(
    db: &Queries,
    auction_ids: &[String],
) -> anyhow::Result<(
    HashMap<String, NFT>,
    HashMap<String, Collection>,
    HashMap<String, Auction>,
)> {
    let auctions = collect_auctions(db, auction_ids).await?;
    let nft_ids = auctions.values().map(|x| x.nft.clone()).collect();
    let (nft, collection) = collect_nft_and_collection(db, &nft_ids).await?;
    Ok((nft, collection, auctions))
}
