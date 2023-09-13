#![deny(
    non_ascii_idents,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_must_use,
    clippy::unwrap_used
)]
#![recursion_limit = "256"]

use api::cfg::ApiConfig;
use api::db::enums::{
    AuctionStatus, DirectBuyState, DirectSellState, NftEventCategory, NftEventType,
};
use api::db::queries::Queries;
use api::handlers;
use api::handlers::auction::{get_auction, get_auction_bids, get_auctions};
use api::handlers::auth::sign_in;
use api::handlers::collection::{
    get_collection, get_collections_by_owner, list_collections, list_collections_simple,
};
use api::handlers::collection_custom::upsert_collection_custom;
use api::handlers::events::{get_events, search_all};
use api::handlers::metrics::get_metrics_summary;
use api::handlers::nft::{
    get_nft, get_nft_direct_buy, get_nft_list, get_nft_price_history, get_nft_random_list,
    get_nft_sell_count, get_nft_top_list,
};
use api::handlers::owner::{
    get_fee, get_owner_bids_in, get_owner_bids_out, get_owner_direct_buy, get_owner_direct_buy_in,
    get_owner_direct_sell,
};
use api::handlers::user::{get_user_by_address, upsert_user};
use api::handlers::*;
use api::model::OrderDirection;
use api::model::*;
use api::schema::Address;
use api::services::auth::AuthService;
use api::token::TokenDict;
use api::usd_price::CurrencyClient;
use handlers::auction::ApiDocAddon as AuctionApiDocAddon;
use handlers::auth::ApiDocAddon as AuthApiDocAddon;
use handlers::collection::ApiDocAddon as CollectionApiDocAddon;
use handlers::events::ApiDocAddon as EventApiDocAddon;
use handlers::metrics::ApiDocAddon as MetricsApiDocAddon;
use handlers::nft::ApiDocAddon as NftApiDocAddon;
use handlers::owner::ApiDocAddon as OwnerApiDocAddon;
use handlers::user::ApiDocAddon as UserApiDocAddon;
use handlers::ApiDocAddon as ModuleApiDocAddon;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use warp::{http::StatusCode, Filter};

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        Address,
        Auction,
        Collection,
        DirectBuy,
        DirectSell,
        Fee,
        DirectBuyState,
        NFT,
        Contract,
        Price,
        AuctionBid,
        DirectSellState,
        AuctionStatus,
        OrderDirection,
        CollectionDetails,
        CollectionDetailsPreviewMeta, NftEventType, NftEventCategory, Attribute
    )),
    info(title="Marketplace API"),
    modifiers(
        &AuctionApiDocAddon,
        &AuthApiDocAddon,
        &CollectionApiDocAddon,
        &MetricsApiDocAddon,
        &EventApiDocAddon,
        &NftApiDocAddon,
        &OwnerApiDocAddon,
        &UserApiDocAddon,
        &ModuleApiDocAddon
    )
)]
struct ApiDoc;

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    dotenv::dotenv().ok();
    stackdriver_logger::init_with_cargo!();
    log::info!("INDEXER-API SERVICE");
    let cfg = ApiConfig::new().expect("Failed to load config");
    let tokens = TokenDict::load()
        .await
        .expect("error loading tokens dictionary");
    let db_pool = cfg.database.init().await.expect("err init database");
    let db_service = Queries::new(Arc::new(db_pool), tokens);
    let auth_service = Arc::new(AuthService::new(
        cfg.auth_token_lifetime,
        cfg.jwt_secret,
        cfg.base_url,
    ));

    CurrencyClient::new(db_service.clone())
        .expect("err initialize currency client")
        .start(std::time::Duration::from_secs(5 * 60)) // 5 minutes
        .await
        .expect("err start currency client");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "authority",
            "user-agent",
            "content-type",
            "authorization",
        ])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let mut cors_headers = warp::http::HeaderMap::new();
    cors_headers.insert(
        "access-control-allow-origin",
        warp::http::HeaderValue::from_static("*"),
    );
    cors_headers.insert(
        "access-control-allow-methods",
        warp::http::HeaderValue::from_static("GET, POST, OPTIONS"),
    );

    let cache_minute = Cache::builder()
        .time_to_live(Duration::from_secs(60))
        .time_to_idle(Duration::from_secs(60))
        .build();

    let cache_5_minutes = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 5))
        .time_to_idle(Duration::from_secs(60 * 5))
        .build();

    let cache_10_sec = Cache::builder()
        .time_to_live(Duration::from_secs(10))
        .time_to_idle(Duration::from_secs(10))
        .build();

    let cache_1_sec = Cache::builder()
        .time_to_live(Duration::from_secs(1))
        .time_to_idle(Duration::from_secs(1))
        .build();

    let api_doc = warp::path("swagger.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc::openapi()));

    let api = warp::any()
        .and(
            warp::options()
                .map(|| StatusCode::NO_CONTENT)
                .with(warp::reply::with::headers(cors_headers))
                .or(api_doc)
                .or(warp::path!("healthz").map(warp::reply))
                .or(get_nft_list(db_service.clone(), cache_10_sec.clone()))
                .or(get_nft_random_list(db_service.clone(), cache_1_sec.clone()))
                .or(get_nft_sell_count(
                    db_service.clone(),
                    cache_5_minutes.clone(),
                ))
                .or(get_nft(db_service.clone()))
                .or(get_nft_top_list(db_service.clone(), cache_minute.clone()))
                .or(get_nft_direct_buy(db_service.clone()))
                .or(get_nft_price_history(db_service.clone()))
                .or(list_collections(
                    db_service.clone(),
                    cache_5_minutes.clone(),
                ))
                .or(list_collections_simple(
                    db_service.clone(),
                    cache_minute.clone(),
                ))
                .or(get_collection(db_service.clone(), cache_5_minutes.clone()))
                .or(get_collections_by_owner(db_service.clone()))
                .or(get_owner_bids_out(db_service.clone()))
                .or(get_owner_bids_in(db_service.clone()))
                .or(get_owner_direct_buy_in(db_service.clone()))
                .or(get_owner_direct_buy(db_service.clone()))
                .or(get_owner_direct_sell(db_service.clone()))
                .or(get_auctions(db_service.clone()))
                .or(get_auction(db_service.clone()))
                .or(get_auction_bids(db_service.clone()))
                .or(get_events(db_service.clone(), cache_minute.clone()))
                .or(get_metrics_summary(
                    db_service.clone(),
                    cache_5_minutes.clone(),
                ))
                .or(list_roots(db_service.clone()))
                .or(search_all(db_service.clone()))
                .or(get_fee(db_service.clone()))
                .or(get_user_by_address(db_service.clone()))
                .or(upsert_user(db_service.clone()))
                .or(upsert_collection_custom(
                    db_service.clone(),
                    auth_service.clone(),
                ))
                .or(sign_in(auth_service.clone())),
        )
        .with(cors);

    let routes = api.with(warp::log("api"));
    log::info!("start http server on {}", cfg.http_address);
    warp::serve(routes).run(cfg.http_address).await;
}
