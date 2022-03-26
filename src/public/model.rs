use crate::Instrument;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Todo:
pub enum Subscription {
    Trades(Instrument),
}

/// Possible public market data types.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum MarketData {
    Trade(Trade),
    Candle,
    Kline,
    OrderBook,
}

/// Normalised public [`Trade`] model.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Trade {
    pub id: String,
    pub exchange: String,
    pub instrument: Instrument,
    pub received_timestamp: DateTime<Utc>,
    pub exchange_timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub amount: Decimal,
    pub direction: Direction,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Direction {
    Long,
    Short
}