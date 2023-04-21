#![allow(clippy::needless_update)]

use opg::*;

pub fn swagger(api_url: &str) -> Opg {
    describe_api! {
        info: {
            title: "NFT Indexer API",
            version: "0.0.1",
            description: "Provides NFT indexer data"
        },
        tags: {
            root,
            auction,
            bid,
            direct_sell,
            direct_buy,
            collection,
            nft,
            search,
            metric,
            event,
        },
        servers: {
            api_url
        },
        paths: {
            ("roots"): {
                GET: {
                    tags: { root },
                    summary: { "Get active roots" },
                    200: Vec<crate::model::Root>,
                }
            },

            ("auctions"): {
                POST: {
                    tags: { auction, bid },
                    summary: "Get NFT auctions list",
                    body: crate::handlers::auction::AuctionsQuery,
                    200: crate::model::VecWith<crate::model::Auction>,
                }
            },

            ("auction"): {
                POST: {
                    tags: { auction, bid },
                    summary: "Get details of auction",
                    body: crate::handlers::auction::AuctionBidsQuery,
                    200: crate::handlers::auction::GetAuctionResult,
                }
            },

            ("auction" / "bids"): {
                POST: {
                    tags: { auction, bid },
                    summary: "Get bids of auction",
                    body: crate::handlers::auction::AuctionBidsQuery,
                    200: crate::model::VecWith<crate::model::AuctionBid>,
                }
            },

            ("collections"): {
                POST: {
                    tags: { collection },
                    summary: "Get collections list",
                    body: crate::handlers::collection::ListCollectionsParams,
                    200: crate::model::VecWithTotal<crate::model::CollectionDetails>,
                }
            },

            ("collections" / "simple"): {
                POST: {
                    tags: { collection },
                    summary: "Get simple collections list",
                    body: crate::handlers::collection::ListCollectionsSimpleParams,
                    200: crate::model::VecWithTotal<crate::model::CollectionSimple>,
                }
            },

            ("collection" / "details"): {
                POST: {
                    tags: { collection },
                    summary: "Get details of collection",
                    body: crate::handlers::collection::CollectionParam,
                    200: crate::model::CollectionDetails,
                }
            },

            ("collections" / "by-owner"): {
                POST: {
                    tags: { collection },
                    summary: "Get collections list by owner",
                    body: crate::handlers::collection::OwnerParam,
                    200: crate::model::VecWithTotal<crate::model::Collection>,
                }
            },

            ("search"): {
                POST: {
                    tags: { search, nft, collection },
                    summary: "Search data",
                    body: String,
                    200: crate::handlers::events::SearchRes,
                }
            },

            ("events"): {
                POST: {
                    tags: { event, nft, collection, auction, direct_sell, direct_buy },
                    summary: "Get activity list",
                    body: crate::handlers::events::EventsQuery,
                    200: crate::model::NftEvents,
                }
            },

            ("metrics" / "summary"): {
                GET: {
                    tags: { metric, collection },
                    summary: "Get summary metrics by collections",
                    parameters: {
                        (query metricsQuery: crate::handlers::metrics::MetricsSummaryQuery): {}
                    },
                    200: crate::model::MetricsSummaryBase,
                }
            },

            ("nft" / "details"): {
                POST: {
                    tags: { nft, auction, direct_buy, direct_sell },
                    summary: "Get NFT details",
                    body: crate::handlers::nft::NFTParam,
                    200: crate::handlers::nft::GetNFTResult,
                }
            },

            ("nft" / "direct" / "buy"): {
                POST: {
                    tags: { nft, direct_buy },
                    summary: "Get direct buys of NFT",
                    body: crate::handlers::nft::NFTParam,
                    200: crate::model::VecWith<crate::model::DirectBuy>,
                }
            },

            ("nft" / "price-history"): {
                POST: {
                    tags: { nft },
                    summary: "Get price history of NFT",
                    body: crate::handlers::nft::NftPriceHistoryQuery,
                    200: Vec<crate::model::NFTPrice>,
                }
            },

            ("nfts" / "top"): {
                POST: {
                    tags: { nft },
                    summary: "Get top NFTs",
                    body: crate::handlers::nft::NFTTopListQuery,
                    200: crate::model::VecWith<crate::model::NFT>,
                }
            },

            ("nfts"): {
                POST: {
                    tags: { nft },
                    summary: "Get NFTs list",
                    body: crate::handlers::nft::NFTListQuery,
                    200: crate::model::VecWith<crate::model::NFT>,
                }
            },

            ("nfts" / "random-buy"): {
                POST: {
                    tags: { nft, direct_buy },
                    summary: "Get random list of NFT direct buys",
                    body: crate::handlers::nft::NFTListRandomBuyQuery,
                    200: crate::model::VecWith<crate::model::NFT>,
                }
            },

            ("nfts" / "sell-count"): {
                GET: {
                    tags: { nft, direct_sell },
                    summary: "Get sell count of NFT",
                    body: crate::handlers::nft::NFTSellCountQuery,
                    200: crate::handlers::nft::NFTSellCountResponse,
                }
            },

            ("owner" / "bids-out"): {
                POST: {
                    tags: { nft, auction, bid },
                    summary: "Get auction bids-out by owner",
                    body: crate::handlers::owner::OwnerBidsOutQuery,
                    200: crate::model::VecWith<crate::model::AuctionBid>,
                }
            },

            ("owner" / "bids-in"): {
                POST: {
                    tags: { nft, auction, bid },
                    summary: "Get auction bids-in by owner",
                    body: crate::handlers::owner::OwnerBidsInQuery,
                    200: crate::model::VecWith<crate::model::AuctionBid>,
                }
            },

            ("owner" / "direct" / "buy"): {
                POST: {
                    tags: { nft, direct_buy },
                    summary: "Get direct buys by owner",
                    body: crate::handlers::owner::OwnerDirectBuyQuery,
                    200: crate::model::VecWith<crate::model::DirectBuy>,
                }
            },

            ("owner" / "direct" / "buy-in"): {
                POST: {
                    tags: { nft, direct_buy },
                    summary: "Get direct buys-in by owner",
                    body: crate::handlers::owner::OwnerDirectBuyQuery,
                    200: crate::model::VecWith<crate::model::DirectBuy>,
                }
            },

            ("owner" / "direct" / "sell"): {
                POST: {
                    tags: { nft, direct_sell },
                    summary: "Get direct sells by owner",
                    body: crate::handlers::owner::OwnerDirectSellQuery,
                    200: crate::model::VecWith<crate::model::DirectSell>,
                }
            },

            ("owner" / "fee"): {
                GET: {
                    tags: { nft, collection },
                    summary: "Get owner's fee",
                    parameters: {
                        (query feeQuery: crate::handlers::owner::OwnerFeeQuery): {}
                    },
                    200: crate::model::OwnerFee,
                }
            },
        }
    }
}

pub fn swagger_yaml(api_url: &str) -> String {
    let api = swagger(api_url);
    serde_yaml::to_string(&api).expect("Failed serializing swagger.yaml")
}

pub fn swagger_json(api_url: &str) -> String {
    let api = swagger(api_url);
    serde_json::to_string(&api).expect("Failed serializing swagger.json")
}
