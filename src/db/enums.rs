use std::fmt::Display;

use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AuctionDeployed = 0,
    AuctionCreated,
    AuctionRootOwnershipTransferred,
    AuctionActive,
    AuctionDeclined,
    AuctionBidPlaced,
    AuctionBidDeclined,
    AuctionCancelled,
    AuctionComplete,

    DirectBuyDeployed,
    DirectBuyDeclined,
    FactoryDirectBuyOwnershipTransferred,
    DirectBuyStateChanged,

    DirectSellDeployed,
    DirectSellDeclined,
    FactoryDirectSellOwnershipTransferred,
    DirectSellStateChanged,

    NftOwnerChanged,
    NftManagerChanged,

    CollectionOwnershipTransferred,

    NftCreated,
    NftBurned,
}


#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Auction = 0,
    DirectBuy,
    DirectSell,
    Nft,
    Collection,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auction_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuctionStatus {
    Active = 0,
    Cancelled,
    Completed,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "direct_sell_state", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DirectSellState {
    Create = 0,
    AwaitNft,
    Active,
    Filled,
    Cancelled,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "direct_buy_state", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DirectBuyState {
    Create = 0,
    AwaitTokens,
    Active,
    Filled,
    Cancelled,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "nft_price_source")]
#[serde(rename_all = "snake_case")]
pub enum NftPriceSource {
    #[sqlx(rename = "auctionBid")]
    AuctionBid = 0,
    #[sqlx(rename = "directBuy")]
    DirectBuy,
    #[sqlx(rename = "directSell")]
    DirectSell,
}

impl Default for AuctionStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl Default for DirectBuyState {
    fn default() -> Self {
        Self::Create
    }
}

impl Default for DirectSellState {
    fn default() -> Self {
        Self::Create
    }
}

impl From<i16> for NftPriceSource {
    fn from(state: i16) -> Self {
        match state {
            0 => Self::AuctionBid,
            1 => Self::DirectBuy,
            2 => Self::DirectSell,
            _ => panic!("Unknown state of DirectSell"),
        }
    }
}

impl From<i16> for AuctionStatus {
    fn from(state: i16) -> Self {
        match state {
            0 => Self::Active,
            1 => Self::Cancelled,
            2 => Self::Completed,
            _ => panic!("Unknown state of DirectSell"),
        }
    }
}

impl From<i16> for EventType {
    fn from(state: i16) -> Self {
        match state {
            0 => Self::AuctionDeployed,
            1 => Self::AuctionCreated,
            2 => Self::AuctionRootOwnershipTransferred,
            3 => Self::AuctionActive,
            4 => Self::AuctionDeclined,
            5 => Self::AuctionBidPlaced,
            6 => Self::AuctionBidDeclined,
            7 => Self::AuctionCancelled,
            8 => Self::AuctionComplete,
        
            9 => Self::DirectBuyDeployed,
            10 => Self::DirectBuyDeclined,
            11 => Self::FactoryDirectBuyOwnershipTransferred,
            12 => Self::DirectBuyStateChanged,
        
            13 => Self::DirectSellDeployed,
            14 => Self::DirectSellDeclined,
            15 => Self::FactoryDirectSellOwnershipTransferred,
            16 => Self::DirectSellStateChanged,
        
            17 => Self::NftOwnerChanged,
            18 => Self::NftManagerChanged,
        
            19 => Self::CollectionOwnershipTransferred,
        
            20 => Self::NftCreated,
            21 => Self::NftBurned,
            _ => panic!("Unknown state of AuctionStatus"),
        }
    }
}

impl From<i16> for DirectSellState {
    fn from(state: i16) -> Self {
        match state {
            0 => Self::Create,
            1 => Self::AwaitNft,
            2 => Self::Active,
            3 => Self::Filled,
            4 => Self::Cancelled,
            5 => Self::Expired,
            _ => panic!("Unknown state of DirectSell"),
        }
    }
}

impl From<i16> for DirectBuyState {
    fn from(state: i16) -> Self {
        match state {
            0 => Self::Create,
            1 => Self::AwaitTokens,
            2 => Self::Active,
            3 => Self::Filled,
            4 => Self::Cancelled,
            5 => Self::Expired,
            _ => panic!("Unknown state of DirectBuy"),
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = serde_json::to_value(self).expect("error serialize EventType");
        let str = val.as_str().expect("not a string");
        f.write_str(str)
    }
}

impl Display for AuctionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = serde_json::to_value(self).expect("error serialize AuctionStatus");
        let str = val.as_str().expect("not a string");
        f.write_str(str)
    }
}

impl Display for DirectBuyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = serde_json::to_value(self).expect("error serialize DirectBuyState");
        let str = val.as_str().expect("not a string");
        f.write_str(str)
    }
}

impl Display for DirectSellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = serde_json::to_value(self).expect("error serialize DirectBuyState");
        let str = val.as_str().expect("not a string");
        f.write_str(str)
    }
}