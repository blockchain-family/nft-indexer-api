use bigdecimal::BigDecimal;

use crate::db::Address;
use crate::handlers::nft::{AttributeFilter, NFTListOrder};

pub struct NftSearchParams<'a> {
    pub owners: &'a [Address],
    pub collections: &'a [Address],
    pub forsale: bool,
    pub auction: bool,
    pub price_from: Option<&'a BigDecimal>,
    pub price_to: Option<&'a BigDecimal>,
    pub price_token: Option<&'a [String]>,
    pub verified: bool,
    pub limit: usize,
    pub offset: usize,
    pub attributes: &'a [AttributeFilter],
    pub order: Option<NFTListOrder>,
    pub with_count: bool,
    pub nft_type: Option<&'a [String]>,
}
