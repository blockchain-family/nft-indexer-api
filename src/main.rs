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
use api::db::queries::Queries;
use api::handlers::*;
use api::services::auth::AuthService;
use api::token::TokenDict;
use api::usd_price::CurrencyClient;
use std::sync::Arc;
use warp::{http::StatusCode, Filter};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    pretty_env_logger::init();
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
        .allow_headers(vec!["authority", "user-agent", "content-type"])
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

    let api = warp::any()
        .and(
            warp::options()
                .map(|| StatusCode::NO_CONTENT)
                .with(warp::reply::with::headers(cors_headers))
                .or(warp::path!("healthz").map(warp::reply))
                .or(get_swagger())
                .or(get_nft(db_service.clone()))
                .or(get_nft_top_list(db_service.clone()))
                .or(get_nft_list(db_service.clone()))
                .or(get_nft_direct_buy(db_service.clone()))
                .or(get_nft_price_history(db_service.clone()))
                .or(list_collections(db_service.clone()))
                .or(list_collections_simple(db_service.clone()))
                .or(get_collection(db_service.clone()))
                .or(get_collections_by_owner(db_service.clone()))
                .or(get_owner_bids_out(db_service.clone()))
                .or(get_owner_bids_in(db_service.clone()))
                .or(get_owner_direct_buy_in(db_service.clone()))
                .or(get_owner_direct_buy(db_service.clone()))
                .or(get_owner_direct_sell(db_service.clone()))
                .or(get_auctions(db_service.clone()))
                .or(get_auction(db_service.clone()))
                .or(get_auction_bids(db_service.clone()))
                .or(get_events(db_service.clone()))
                .or(get_metrics_summary(db_service.clone()))
                .or(list_roots(db_service.clone()))
                .or(search_all(db_service.clone()))
                .or(get_fee(db_service.clone()))
                .or(get_user_by_address(db_service.clone()))
                .or(sign_in(auth_service.clone())),
        )
        .with(cors);

    let routes = api.with(warp::log("api"));
    log::info!("start http server on {}", cfg.http_address);
    warp::serve(routes).run(cfg.http_address).await;
}
