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

use api::cfg::ApiConfig;
use api::db::Queries;
use api::handlers::*;
use api::token::TokenDict;
use api::usd_price::CurrencyClient;
use std::sync::Arc;
use warp::{http::StatusCode, Filter};

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    pretty_env_logger::init();
    log::info!("INDEXER-API SERVICE");
    let cfg = ApiConfig::new().expect("Failed to load config");
    log::info!(
        "BACKEND_API_USER={}",
        std::env::var("BACKEND_API_USER").expect("err read BACKEND_API_USER env")
    );

    let tokens = TokenDict::load()
        .await
        .expect("error loading tokens dictionary");
    let db_pool = cfg.database.init().await.expect("err init database");
    let service = Queries::new(Arc::new(db_pool), tokens);

    CurrencyClient::new(service.clone())
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
                .or(get_nft(service.clone()))
                .or(get_nft_top_list(service.clone()))
                .or(get_nft_list(service.clone()))
                .or(get_nft_direct_buy(service.clone()))
                .or(get_nft_price_history(service.clone()))
                .or(list_collections(service.clone()))
                .or(list_collections_simple(service.clone()))
                .or(get_collection(service.clone()))
                .or(get_collections_by_owner(service.clone()))
                .or(get_owner_bids_out(service.clone()))
                .or(get_owner_bids_in(service.clone()))
                .or(get_owner_direct_buy_in(service.clone()))
                .or(get_owner_direct_buy(service.clone()))
                .or(get_owner_direct_sell(service.clone()))
                .or(get_auctions(service.clone()))
                .or(get_auction(service.clone()))
                .or(get_auction_bids(service.clone()))
                .or(get_events(service.clone()))
                .or(get_metrics_summary(service.clone()))
                .or(list_roots(service.clone()))
                .or(search_all(service.clone()))
                .or(get_fee(service.clone())),
        )
        .with(cors);

    let routes = api.with(warp::log("api"));
    log::info!("start http server on {}", cfg.http_address);
    warp::serve(routes).run(cfg.http_address).await;
}
