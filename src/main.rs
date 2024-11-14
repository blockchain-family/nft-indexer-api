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
use api::db::enums::{AuctionStatus, DirectBuyState, DirectSellState, NftEventType};
use api::db::queries::Queries;
use api::handlers;
use api::handlers::auction::*;
use api::handlers::auth::*;
use api::handlers::collection::*;
use api::handlers::collection_custom::*;
use api::handlers::events::*;
use api::handlers::metadata::*;
use api::handlers::metrics::*;
use api::handlers::nft::*;
use api::handlers::owner::*;
use api::handlers::user::*;
use api::handlers::{requests::Period, *};
use api::model::OrderDirection;
use api::model::*;
use api::schema::Address;
use api::services::auth::AuthService;
use api::token::TokenDict;

use api::usd_price::CurrencyClient;
use handlers::auction::ApiDocAddon as AuctionApiDocAddon;
use handlers::auth::ApiDocAddon as AuthApiDocAddon;
use handlers::collection::ApiDocAddon as CollectionApiDocAddon;
use handlers::collection_custom::ApiDocAddon as CollectionCustomAddon;
use handlers::events::ApiDocAddon as EventApiDocAddon;
use handlers::metadata::ApiDocAddon as MetadataApiDocAddon;
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
        CollectionDetailsPreviewMeta, NftEventType, Attribute,
        CollectionEvaluationList,
        CollectionEvaluation,
        Period,
    )),
    info(title = "Marketplace API"),
    modifiers(
        &AuctionApiDocAddon,
        &AuthApiDocAddon,
        &CollectionApiDocAddon,
        &MetricsApiDocAddon,
        &EventApiDocAddon,
        &NftApiDocAddon,
        &OwnerApiDocAddon,
        &UserApiDocAddon,
        &ModuleApiDocAddon,
        &CollectionCustomAddon,
        &MetadataApiDocAddon
    )
)]
struct ApiDoc;

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    dotenv::dotenv().ok();
    stackdriver_logger::init_with_cargo!();
    log::info!("INDEXER-API SERVICE");
    let cfg = ApiConfig::new();
    let tokens = TokenDict::load(&cfg.token_manifest_path)
        .await
        .expect("error loading tokens dictionary");
    let db_pool = cfg.database.init().await.expect("err init database");
    let db_service = Queries::new(Arc::new(db_pool), cfg.main_token.clone(), tokens);
    let auth_service = Arc::new(AuthService::new(
        cfg.auth_token_lifetime,
        cfg.jwt_secret,
        cfg.service_name,
    ));

    CurrencyClient::new(db_service.clone(), cfg.main_token, cfg.dex_url)
        .expect("err initialize currency client")
        .start(Duration::from_secs(5 * 60)) // 5 minutes
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

    let mut cors_headers = http::HeaderMap::new();
    cors_headers.insert(
        "access-control-allow-origin",
        http::HeaderValue::from_static("*"),
    );
    cors_headers.insert(
        "access-control-allow-methods",
        http::HeaderValue::from_static("GET, POST, OPTIONS"),
    );

    let cache_minute = Cache::builder()
        .time_to_live(Duration::from_secs(60))
        .build();

    let cache_5_minutes = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 5))
        .build();

    let cache_10_sec = Cache::builder()
        .time_to_live(Duration::from_secs(10))
        .build();

    let cache_1_sec = Cache::builder()
        .time_to_live(Duration::from_secs(1))
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
                .or(get_nft_for_banner(db_service.clone(), cache_minute.clone()))
                .or(get_nft_direct_buy(db_service.clone()))
                .or(get_nft_price_history(db_service.clone()))
                .boxed()
                .or(list_collections(db_service.clone(), cache_minute.clone()))
                .or(list_collections_simple(
                    db_service.clone(),
                    cache_minute.clone(),
                ))
                .or(list_collections_evaluation(
                    db_service.clone(),
                    cache_5_minutes.clone(),
                ))
                .or(get_collection(db_service.clone(), cache_1_sec.clone()))
                .or(get_collections_by_owner(db_service.clone()))
                .boxed()
                .or(get_nft_types(db_service.clone(), cache_5_minutes.clone()))
                .or(get_owner_bids_out(db_service.clone()))
                .or(get_owner_bids_in(db_service.clone()))
                .or(get_owner_direct_buy_in(db_service.clone()))
                .or(get_owner_direct_buy(db_service.clone()))
                .or(get_owner_direct_sell(db_service.clone()))
                .boxed()
                .or(get_auctions(db_service.clone()))
                .or(get_auction(db_service.clone()))
                .or(get_auction_bids(db_service.clone()))
                .or(get_events(db_service.clone(), cache_10_sec.clone()))
                .or(get_metrics_summary(
                    db_service.clone(),
                    cache_minute.clone(),
                ))
                .boxed()
                .or(list_roots(db_service.clone()))
                .or(search_all(db_service.clone()))
                .or(get_fee(db_service.clone()))
                .or(get_user_by_address(db_service.clone()))
                .or(upsert_user(db_service.clone()))
                .or(upsert_collection_custom(
                    db_service.clone(),
                    auth_service.clone(),
                ))
                .or(update_metadata(cfg.indexer_api_url))
                .or(sign_in(auth_service.clone()))
                .or(get_nfts_price_range(db_service.clone()))
                .or(get_my_best_offer(db_service.clone(), auth_service.clone())),
        )
        .with(cors);

    let routes = api.with(warp::log("api"));
    log::info!("start http server on {}", cfg.http_address);
    warp::serve(routes).run(cfg.http_address).await;
}
