use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Todo:
pub mod futures;

/// Type alias to communicate a `String` is a Binance stream identifier (eg/ btcusdt@aggTrade)
pub type BinanceStreamId = String;

/// Binance Message variants that could be received over [`WebSocket`].
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum BinanceMessage {
    Subscribed(BinanceSubscribed),
    Trade(BinanceTrade)
}

/// Binance specific subscription confirmation message.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct BinanceSubscribed {
    result: Option<Vec<String>>,
    id: u32,
}

/// Binance specific Trade message.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct BinanceTrade {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "T")]
    trade_ts: u64,
    #[serde(rename = "a")]
    id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "m")]
    buyer_is_maker: bool,
}