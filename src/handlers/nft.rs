use crate::db::queries::Queries;
use crate::db::{MetaRoyalty, NftDetails, NftForBanner};
use crate::handlers::calculate_hash;
use crate::model::{DirectBuy, NFTPrice, NftTrait, NftsPriceRange, OrderDirection, VecWith, NFT};
use crate::{
    api_doc_addon, catch_empty, catch_error_500,
    db::{Address, DirectBuyState},
    model::{Auction, Collection, DirectSell},
    response,
};

use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;
use tokio::join;
use warp::http::StatusCode;
use warp::Filter;

use crate::db::query_params::nft::NftSearchParams;
use crate::handlers::auction::collect_auctions;
use crate::handlers::collection::collect_collections;
use crate::schema::VecWithDirectBuy;
use crate::schema::VecWithNFT;
use utoipa::OpenApi;
use utoipa::ToSchema;

#[derive(OpenApi)]
#[openapi(
    paths(get_nft, get_nft_direct_buy, get_nft_price_history, get_nft_list, get_nft_top_list, get_nft_random_list, get_nft_types, get_nft_for_banner, get_nfts_price_range),
    components(schemas(
        NFTParam,
        GetNFTResult,
        NftTrait,
        NftPriceHistoryQuery,
        NFTPrice,
        VecWithNFT,
        NFTListOrder,
        AttributeFilter,
        NFTListQuery,
        PriceHistoryScale,
        VecWithDirectBuy,
        NFTListOrderField,
        NFTTopListQuery,
        MetaRoyalty,
        NFTListRandomBuyQuery,
        NftForBanner,
        NftPriceRangeParams,
        NftsPriceRange
    )),
    tags(
        (name = "nft", description = "NFT handlers"),
    )
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nft/details",
    request_body(content = NFTParam, description = "Get NFT"),
    responses(
        (status = 200, body = GetNFTResult),
        (status = 500),
    ),
)]
pub fn get_nft(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
    let nft = catch_error_500!(db.get_nft_details(&param.nft).await);
    let nft = catch_empty!(nft, "not found");
    let collections_ids = match &nft.collection {
        Some(c) => vec![c.clone()],
        None => vec![],
    };

    let collection = catch_error_500!(collect_collections(&db, &collections_ids).await);

    let mut auction = HashMap::default();
    if let Some(ref auction_id) = nft.auction {
        let a = catch_error_500!(db.get_nft_auction(auction_id).await);
        if let Some(a) = a {
            auction.insert(auction_id.clone(), Auction::from_db(&a, &db.tokens));
        }
    };

    let mut direct_sell = HashMap::default();
    if let Some(ref direct_sell_id) = nft.forsale {
        let a = catch_error_500!(db.get_direct_sell(direct_sell_id).await);
        if let Some(a) = a {
            direct_sell.insert(direct_sell_id.clone(), DirectSell::from_db(&a, &db.tokens));
        }
    };

    let mut direct_buy = HashMap::default();
    let nft_addr = nft.address.clone().unwrap_or_default();
    let mut list = catch_error_500!(
        db.list_nft_direct_buy(&nft_addr, &[DirectBuyState::Active], 100, 0)
            .await
    );

    for x in list.drain(..) {
        direct_buy.insert(x.address.clone(), DirectBuy::from_db(&x, &db.tokens));
    }

    let traits = db.get_traits(&nft_addr).await;
    let traits = match traits {
        Ok(traits) => traits,
        Err(e) => {
            log::error!("Load traits error {e:?}");
            vec![]
        }
    };

    let traits: Vec<NftTrait> = traits.into_iter().map(NftTrait::from).collect();

    let ret = GetNFTResult {
        nft: NFT::from_db(nft),
        collection,
        auction,
        direct_buy,
        direct_sell,
        traits,
    };

    response!(&ret)
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetNFTResult {
    pub nft: NFT,
    pub collection: HashMap<Address, Collection>,
    pub auction: HashMap<Address, Auction>,
    #[serde(rename = "directSell")]
    pub direct_sell: HashMap<Address, DirectSell>,
    #[serde(rename = "directBuy")]
    pub direct_buy: HashMap<Address, DirectBuy>,
    pub traits: Vec<NftTrait>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct NFTParam {
    pub nft: Address,
    pub status: Option<Vec<DirectBuyState>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nft/direct/buy",
    request_body(content = NFTParam, description = "Get NFT direct buy"),
    responses(
        (status = 200, body = VecWithDirectBuy),
        (status = 500),
    ),
)]
pub fn get_nft_direct_buy(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
    let list = catch_error_500!(db.list_nft_direct_buy(&nft, status, limit, offset).await);

    let count = list.first().map(|it| it.cnt).unwrap_or_default();
    let ret: Vec<DirectBuy> = list
        .iter()
        .map(|x| DirectBuy::from_db(x, &db.tokens))
        .collect();
    let nft_ids = ret.iter().map(|x| x.nft.clone()).collect();

    let (nft, collection) = catch_error_500!(collect_nft_and_collection(&db, &nft_ids).await);
    let ret = VecWithDirectBuy {
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
    post,
    tag = "nft",
    path = "/nft/price-history",
    request_body(content = NftPriceHistoryQuery, description = "Get NFT price history"),
    responses(
        (status = 200, body = Vec<NFTPrice>),
        (status = 500),
    ),
)]
pub fn get_nft_price_history(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
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
    let from = NaiveDateTime::from_timestamp_opt(query.from, 0).expect("Failed to get datetime");
    let to = NaiveDateTime::from_timestamp_opt(query.to, 0).expect("Failed to get datetime");
    let list = catch_error_500!(db.list_nft_price_history(&query.nft, from, to).await);
    let ret: Vec<NFTPrice> = list.into_iter().map(NFTPrice::from_db).collect();
    response!(&ret)
}

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nfts/top",
    request_body(content = NFTTopListQuery, description = "Get NFT top list"),
    responses(
        (status = 200, body = VecWithNFT),
        (status = 500),
    ),
)]
pub fn get_nft_top_list(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "top")
        .and(warp::post())
        .and(warp::body::json::<NFTTopListQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_top_list_handler)
}
#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct NFTTopListQuery {
    pub from: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Deserialize, Serialize, Hash)]
struct NFTTopListQueryCache {
    pub limit: i64,
    pub offset: i64,
}

/// POST /nfts/
#[utoipa::path(
    post,
    tag = "nft",
    path = "/nfts",
    request_body(content = NFTListQuery, description = "NFT list"),
    responses(
        (status = 200, body = VecWithNFT),
        (status = 500),
    ),
)]
pub fn get_nft_list(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts")
        .and(warp::post())
        .and(warp::body::json::<NFTListQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_list_handler)
}

pub async fn get_nft_list_handler(
    params: NFTListQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&params);
    let cached_value = cache.get(&hash);

    let response;
    match cached_value {
        None => {
            let with_count = params.with_count.unwrap_or(false);
            let limit = params.limit.unwrap_or(100);
            let final_limit = match with_count {
                true => limit,
                false => limit + 1,
            };

            let price_from = catch_error_500!(params
                .price_from
                .map(|p| BigDecimal::from_str(&p))
                .transpose());

            let price_to = catch_error_500!(params
                .price_to
                .map(|p| BigDecimal::from_str(&p))
                .transpose());

            let search_params = &NftSearchParams {
                owners: params.owners.as_deref().unwrap_or(&[]),
                collections: params.collections.as_deref().unwrap_or(&[]),
                forsale: params.forsale.unwrap_or(false),
                auction: params.auction.unwrap_or(false),
                price_from: price_from.as_ref(),
                price_to: price_to.as_ref(),
                verified: params.verified.unwrap_or(true),
                limit: final_limit,
                offset: params.offset.unwrap_or_default(),
                attributes: params.attributes.as_deref().unwrap_or_default(),
                order: params.order,
                with_count,
                nft_type: params.nft_type.as_ref(),
            };

            let list = if search_params.verified {
                catch_error_500!(db.nft_search_verified(search_params,).await)
            } else {
                catch_error_500!(db.nft_search(search_params,).await)
            };

            let mut r = catch_error_500!(make_nfts_response(list, db).await);
            if !with_count {
                if r.items.len() < final_limit {
                    r.count = (r.items.len() + search_params.offset) as i64
                } else {
                    r.items.pop();
                    r.count = (r.items.len() + search_params.offset + 1) as i64;
                }
            }

            response = r;
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&response)
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NFTListRandomBuyQuery {
    pub max_price: i64,
    pub limit: i32,
}

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nfts/random-buy",
    request_body(content = NFTListRandomBuyQuery, description = "NFT Random buy list"),
    responses(
        (status = 200, body = VecWithNFT),
        (status = 500),
    ),
)]
pub fn get_nft_random_list(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "random-buy")
        .and(warp::post())
        .and(warp::body::json::<NFTListRandomBuyQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_random_list_handler)
}

pub async fn get_nft_random_list_handler(
    params: NFTListRandomBuyQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&params);
    let cached_value = cache.get(&hash);

    let response;
    match cached_value {
        None => {
            let mut limit = params.limit;
            if limit > 30 {
                limit = 30
            }
            let max_price = params.max_price;

            let list = catch_error_500!(db.nft_random_buy(max_price, limit).await);
            let mut r = catch_error_500!(make_nfts_response(list, db).await);

            r.count = r.items.len() as i64;

            response = r;
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&response)
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NFTSellCountQuery {
    pub max_price: i64,
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NFTSellCountResponse {
    pub count: i64,
    pub timestamp: i64,
}

/// GET /nfts/sell-count
pub fn get_nft_sell_count(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "sell-count")
        .and(warp::get())
        .and(warp::body::json::<NFTSellCountQuery>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_sell_count_handler)
}

pub async fn get_nft_sell_count_handler(
    params: NFTSellCountQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&params);
    let cached_value = cache.get(&hash);
    let response;
    match cached_value {
        None => {
            let max_price = params.max_price;
            let sell_count =
                catch_error_500!(db.nft_sell_count(max_price).await).unwrap_or_default();
            response = NFTSellCountResponse {
                count: sell_count,
                timestamp: chrono::offset::Utc::now().naive_utc().timestamp(),
            };
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&response)
}

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nft/banner",
    responses(
        (status = 200, body = Vec<NftForBanner>),
        (status = 500),
    ),
)]
pub fn get_nft_for_banner(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nft" / "banner")
        .and(warp::post())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_for_banner_handler)
}

pub async fn get_nft_for_banner_handler(
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let hash = calculate_hash(&"nft/banner".to_string());
    let cached_value = cache.get(&hash);

    let nft_for_banner: Vec<NftForBanner>;
    match cached_value {
        None => {
            nft_for_banner = catch_error_500!(db.nft_get_for_banner().await);
            let value_for_cache = serde_json::to_value(nft_for_banner.clone())
                .expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            nft_for_banner =
                serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&nft_for_banner)
}

pub async fn get_nft_top_list_handler(
    params: NFTTopListQuery,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let params_cache = NFTTopListQueryCache {
        limit: params.limit,
        offset: params.offset,
    };
    let hash = calculate_hash(&params_cache);
    let cached_value = cache.get(&hash);

    let response;
    match cached_value {
        None => {
            let from =
                NaiveDateTime::from_timestamp_opt(params.from, 0).expect("Failed to get datetime");
            let list = catch_error_500!(db.nft_top_search(from, params.limit, params.offset).await);
            response = catch_error_500!(make_nfts_response(list, db).await);
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    Ok(Box::from(warp::reply::with_status(
        warp::reply::json(&response),
        StatusCode::OK,
    )))
}

#[derive(Clone, Deserialize, Serialize, Hash)]
struct NFTTypeCache {
    pub verified_type: bool,
}

#[utoipa::path(
    get,
    tag = "nft",
    path = "/nfts/types",
    responses(
        (status = 200, body =  Vec<String>),
        (status = 500),
    ),
)]
pub fn get_nft_types(
    db: Queries,
    cache: Cache<u64, Value>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "types")
        .and(warp::post())
        .and(warp::body::json::<NFTType>())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || cache.clone()))
        .and_then(get_nft_types_handler)
}

pub async fn get_nft_types_handler(
    params: NFTType,
    db: Queries,
    cache: Cache<u64, Value>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let verified_flag = params.verified.unwrap_or(true);
    let params_cache = NFTTypeCache {
        verified_type: verified_flag,
    };
    let hash = calculate_hash(&params_cache);
    let cached_value = cache.get(&hash);

    let response: Vec<String>;
    match cached_value {
        None => {
            let list_of_types = catch_error_500!(db.nft_get_types(verified_flag).await);
            response = list_of_types.iter().map(|x| x.mimetype.clone()).collect();
            let value_for_cache =
                serde_json::to_value(response.clone()).expect("Failed serializing cached value");
            cache.insert(hash, value_for_cache).await;
        }
        Some(cached_value) => {
            response = serde_json::from_value(cached_value).expect("Failed parsing cached value")
        }
    }

    response!(&response)
}

async fn make_nfts_response(list: Vec<NftDetails>, db: Queries) -> anyhow::Result<VecWith<NFT>> {
    let count = match list.first() {
        None => 0,
        Some(first) => first.total_count,
    };

    let ret: Vec<NFT> = list.iter().map(|it| NFT::from_db(it.clone())).collect();

    let collection_ids = ret.iter().map(|x| x.collection.clone()).collect();
    let collection = collect_collections(&db, &collection_ids);

    let auction_ids: Vec<String> = list.iter().filter_map(|x| x.auction.clone()).collect();
    let auction = collect_auctions(&db, &auction_ids);

    let direct_sell_ids: Vec<String> = list.iter().filter_map(|x| x.forsale.clone()).collect();
    let direct_sell = collect_direct_sell(&db, &direct_sell_ids);

    let direct_buy_ids: Vec<String> = list.iter().filter_map(|x| x.best_offer.clone()).collect();
    let direct_buy = collect_direct_buy(&db, &direct_buy_ids);

    let (collection_result, auction_result, direct_sell_result, direct_buy_result) =
        join!(collection, auction, direct_sell, direct_buy,);

    Ok(VecWith {
        count,
        items: ret,
        collection: Some(collection_result.context("Failed to get collection_result")?),
        nft: None,
        auction: Some(auction_result.context("Failed to get auction_result")?),
        direct_buy: Some(direct_buy_result.context("Failed to get direct_buy_result")?),
        direct_sell: Some(direct_sell_result.context("Failed to get direct_sell_result")?),
    })
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct AttributeFilter {
    #[serde(rename = "traitType")]
    pub trait_type: String,
    #[serde(rename = "traitValues")]
    pub trait_values: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct NFTListQuery {
    pub owners: Option<Vec<String>>,
    pub collections: Option<Vec<String>>,
    #[serde(rename = "priceFrom")]
    pub price_from: Option<String>,
    #[serde(rename = "priceTo")]
    pub price_to: Option<String>,
    #[serde(rename = "priceToken")]
    pub price_token: Option<String>,
    pub forsale: Option<bool>,
    pub auction: Option<bool>,
    pub verified: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub attributes: Option<Vec<AttributeFilter>>,
    pub order: Option<NFTListOrder>,
    #[serde(rename = "withCount")]
    pub with_count: Option<bool>,
    #[serde(rename = "nftType")]
    pub nft_type: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct NFTType {
    pub verified: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub enum NFTListOrderField {
    #[serde(rename = "floorPriceUsd")]
    FloorPriceUsd,
    #[serde(rename = "dealPriceUsd")]
    DealPriceUsd,
    #[serde(rename = "name")]
    Name,
}

impl Display for NFTListOrderField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NFTListOrderField::FloorPriceUsd => write!(f, "floor_price_usd"),
            NFTListOrderField::DealPriceUsd => write!(f, "deal_price_usd"),
            NFTListOrderField::Name => write!(f, "name"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct NFTListOrder {
    pub field: NFTListOrderField,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum PriceHistoryScale {
    #[serde(rename = "h")]
    Hours,
    #[serde(rename = "d")]
    Days,
}

impl Display for PriceHistoryScale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceHistoryScale::Days => write!(f, "day"),
            PriceHistoryScale::Hours => write!(f, "hour"),
        }
    }
}

impl Default for PriceHistoryScale {
    fn default() -> Self {
        Self::Days
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NftPriceHistoryQuery {
    pub nft: Address,
    pub scale: Option<PriceHistoryScale>,
    pub from: i64,
    pub to: i64,
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

#[allow(clippy::ptr_arg)]
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

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct NftPriceRangeParams {
    pub owners: Vec<Address>,
    pub collections: Vec<Address>,
    pub verified: Option<bool>,
    pub attributes: Vec<AttributeFilter>,
}

#[utoipa::path(
    post,
    tag = "nft",
    path = "/nfts/price-range",
        responses(
            (status = 200, body =  NftsPriceRange),
            (status = 500),
        ),
)]
pub fn get_nfts_price_range(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("nfts" / "price-range")
        .and(warp::post())
        .and(warp::body::json::<NftPriceRangeParams>())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_nfts_price_range_handler)
}

pub async fn get_nfts_price_range_handler(
    query: NftPriceRangeParams,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let prices = catch_error_500!(
        db.nft_price_range_verified(
            &query.collections,
            &query.attributes,
            &query.owners,
            query.verified.unwrap_or(true)
        )
        .await
    );

    if let Some(prices) = prices {
        let response: NftsPriceRange = prices.into();
        return response!(&response);
    }

    response!(None::<Option<NftsPriceRange>>)
}
