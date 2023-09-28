use crate::db::Address;
use crate::handlers::nft::{AttributeFilter, NFTListOrder};
use bigdecimal::BigDecimal;

pub struct NftSearchParams<'a> {
    pub owners: &'a [Address],
    pub collections: &'a [Address],
    pub forsale: Option<bool>,
    pub auction: Option<bool>,
    pub price_from: Option<&'a BigDecimal>,
    pub price_to: Option<&'a BigDecimal>,
    pub verified: Option<bool>,
    pub limit: usize,
    pub offset: usize,
    pub _attributes: &'a [AttributeFilter],
    pub order: Option<&'a NFTListOrder>,
    pub with_count: bool,
    pub nft_type: Option<&'a String>,
}
