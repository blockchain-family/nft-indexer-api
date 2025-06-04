use api::cfg::ApiConfig;
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
use api::handlers::service::*;
use api::handlers::swagger::*;
use api::handlers::user::*;
use api::services::auth::AuthService;
use api::token::TokenDict;
use api::usd_price::CurrencyClient;
use axum::{
    Router,
    http::{Method, StatusCode},
    routing::{get, post},
};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    stackdriver_logger::init_with_cargo!();
    log::info!("INDEXER-API SERVICE");
    let cfg = ApiConfig::new();
    let tokens = TokenDict::load(&cfg.token_manifest_path)
        .await
        .expect("error loading tokens dictionary");
    let db_pool = cfg.database.init().await.expect("err init database");
    let db_service = Queries::new(Arc::new(db_pool), cfg.main_token.clone(), tokens);
    let auth_service = AuthService::new(cfg.auth_token_lifetime, cfg.jwt_secret, cfg.service_name);

    CurrencyClient::new(db_service.clone(), cfg.main_token, cfg.dex_url)
        .expect("err initialize currency client")
        .start(Duration::from_secs(5 * 60)) // 5 minutes
        .await
        .expect("err start currency client");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::USER_AGENT,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS]);

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

    let state = Arc::new(handlers::HttpState {
        db: db_service,
        auth_service,
        cache_minute,
        cache_5_minutes,
        cache_10_sec,
        cache_1_sec,
        indexer_api_url: cfg.indexer_api_url,
    });

    let service_routes = Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .merge(swagger_json(&cfg.base_url))
        .merge(swagger_yaml(&cfg.base_url))
        .merge(swagger_ui(&cfg.base_url));

    let app_routes = Router::new()
        // NFT endpoints
        .route("/nfts", post(get_nft_list))
        .route("/nfts/random-buy", post(get_nft_random_list))
        .route("/nfts/sell-count", post(get_nft_sell_count))
        .route("/nft/details", post(get_nft))
        .route("/nfts/top", post(get_nft_top_list))
        .route("/nft/banner", post(get_nft_for_banner))
        .route("/nft/direct/buy", post(get_nft_direct_buy))
        .route("/nft/price-history", post(get_nft_price_history))
        .route("/nfts/types", post(get_nft_types))
        .route("/nfts/price-range", post(get_nfts_price_range))
        .route("/nft/my-best-offer", get(get_my_best_offer))
        // Collection endpoints
        .route("/collections", post(list_collections))
        .route("/collections/simple", post(list_collections_simple))
        .route("/collections/evaluation", post(list_collections_evaluation))
        .route("/collection/details", post(get_collection))
        .route("/collections/by-owner", post(get_collections_by_owner))
        .route("/collections-custom", post(upsert_collection_custom))
        // Owner endpoints
        .route("/owner/bids-out", post(get_owner_bids_out))
        .route("/owner/bids-in", post(get_owner_bids_in))
        .route("/owner/direct/buy", post(get_owner_direct_buy))
        .route("/owner/direct/buy-in", post(get_owner_direct_buy_in))
        .route("/owner/direct/sell", post(get_owner_direct_sell))
        .route("/owner/fee", get(get_fee))
        // Auction endpoints
        .route("/auctions", post(get_auctions))
        .route("/auction", post(get_auction))
        .route("/auction/bids", post(get_auction_bids))
        // Event endpoints
        .route("/events", post(get_events))
        .route("/search", post(search_all))
        // Metrics endpoints
        .route("/metrics/summary", get(get_metrics_summary))
        // User endpoints
        .route("/user/{address}", get(get_user_by_address))
        .route("/user", post(upsert_user))
        // Auth endpoints
        .route("/user/sign_in", post(sign_in))
        // Metadata endpoints
        .route("/update-metadata", post(update_metadata))
        // Service endpoints
        .route("/roots", get(list_roots))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.http_address)
        .await
        .unwrap();

    log::info!("start http server on {}", cfg.http_address);

    axum::serve(listener, service_routes.merge(app_routes))
        .await
        .unwrap();
}
