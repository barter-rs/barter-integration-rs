use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Todo:
pub mod futures;

/// Binance Message variants that could be received over [`WebSocket`].
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
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
    trade_ts: i64,
    #[serde(rename = "a")]
    trade_id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "m")]
    buyer_is_maker: bool,
}