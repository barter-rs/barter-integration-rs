use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use crate::Instrument;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use crate::public::explore::{StreamConfig, StreamKind};

/// Todo:
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Subscription {
    Trades(Instrument),
}

impl From<StreamConfig> for Subscription {
    fn from(stream: StreamConfig) -> Self {
        match stream.kind {
            StreamKind::Trade => Self::Trades(stream.instrument)
        }
    }
}

/// Todo:
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MarketEvent {
    pub sequence: Sequence,
    pub timestamp: DateTime<Utc>,
    pub data: MarketData,
}

impl MarketEvent {
    pub fn new(sequence: Sequence, data: MarketData) -> Self {
        Self {
            sequence,
            timestamp: Utc::now(),
            data
        }
    }
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
    pub quantity: Decimal,
    pub direction: Direction,
}

/// Todo:
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Direction {
    Long,
    Short
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct StreamMeta {
    pub sequence: Sequence,
    pub subscription: Subscription,
}

impl StreamMeta {
    pub fn new(subscription: Subscription) -> Self {
        Self {
            sequence: Sequence(0),
            subscription
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Sequence(pub u64);

impl Display for Sequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Sequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<u64> for Sequence {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Deref for Sequence {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Sequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for Sequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        u64::deserialize(deserializer).map(Sequence)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize)]
pub struct StreamId(pub String);

impl Display for StreamId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for StreamId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for StreamId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for StreamId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        String::deserialize(deserializer).map(StreamId)
    }
}

impl<S> From<S> for StreamId where S: Into<String> {
    fn from(input: S) -> Self {
        Self(input.into())
    }
}