#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
)]

///! # Barter-Integration

use crate::socket::{
    error::SocketError,
    protocol::websocket::WsMessage,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};
use serde::{Deserialize, Deserializer, Serialize};

/// Contains an `ExchangeSocket` capable of acting as a `Stream` and `Sink` for a given remote
/// server.
pub mod socket;

/// Barter representation of an `Instrument`. Used to uniquely identify a `base_quote` pair, and it's
/// associated instrument type.
///
/// eg/ Instrument { base: "btc", quote: "usdt", kind: Spot }
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    #[serde(alias = "instrument_type")]
    pub kind: InstrumentKind,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}_{}, {}", self.base, self.quote, self.kind)
    }
}

impl<S> From<(S, S, InstrumentKind)> for Instrument
where
    S: Into<Symbol>,
{
    fn from((base, quote, kind): (S, S, InstrumentKind)) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind
        }
    }
}

impl Instrument {
    /// Constructs a new [`Instrument`] using the provided configuration.
    pub fn new<S>(base: S, quote: S, kind: InstrumentKind) -> Self
    where
        S: Into<Symbol>
    {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind,
        }
    }

    /// Generates a unique identifier for an [`Instrument`] being traded on the provided exchange.
    pub fn to_id(&self, exchange: &str) -> InstrumentId {
        InstrumentId::new(self, exchange)
    }
}

/// Defines the type of [`Instrument`] which is being traded on a given `base_quote` market.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentKind {
    Spot,
    FuturePerpetual,
    FutureQuarterly,
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            InstrumentKind::Spot => "spot",
            InstrumentKind::FuturePerpetual => "future_perpetual",
            InstrumentKind::FutureQuarterly => "future_quarterly",
        })
    }
}

/// Barter new type representing a currency symbol `String` identifier.
///
/// eg/ "btc", "eth", "usdt", etc
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Symbol(String);

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        String::deserialize(deserializer).map(Symbol::new)
    }
}

impl<S> From<S> for Symbol
where
    S: Into<String>
{
    fn from(input: S) -> Self {
        Symbol::new(input)
    }
}

impl Symbol {
    /// Construct a new [`Symbol`] new type using the provided `Into<Symbol>` value.
    pub fn new<S>(input: S) -> Self where S: Into<String> {
        Self(input.into().to_lowercase())
    }
}

/// Barter new type representing a unique `String` identifier for an [`Instrument`] being traded
/// on the provided exchange.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct InstrumentId(String);

impl Debug for InstrumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for InstrumentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for InstrumentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for InstrumentId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(InstrumentId)
    }
}

impl InstrumentId {
    /// Construct a unique `String` identifier for an [`Instrument`].
    pub fn new(instrument: &Instrument, exchange: &str) -> Self {
        Self(format!("{}_{}_{}_{}", exchange, instrument.base, instrument.quote, instrument.kind).to_lowercase())
    }
}

/// Barter [`Subscription`] used to subscribe to a market [`StreamKind`] for a particular
/// [`Instrument`].
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Subscription {
    pub instrument: Instrument,
    pub kind: StreamKind,
}

impl Debug for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.kind, self.instrument)
    }
}

impl Display for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<I> From<(I, StreamKind)> for Subscription
where
    I: Into<Instrument>
{
    fn from((instrument, kind): (I, StreamKind)) -> Self {
        Self {
            instrument: instrument.into(),
            kind
        }
    }
}

impl<S> From<(S, S, InstrumentKind, StreamKind)> for Subscription
where
    S: Into<Symbol>
{
    fn from((base, quote, instrument, stream): (S, S, InstrumentKind, StreamKind)) -> Self {
        Self {
            instrument: Instrument::from((base, quote, instrument)),
            kind: stream
        }
    }
}

impl Subscription {
    /// Constructs a new [`Subscription`] using the provided configuration.
    pub fn new<I>(instrument: I, kind: StreamKind) -> Self
    where
        I: Into<Instrument>
    {
        Self {
            instrument: instrument.into(),
            kind
        }
    }
}

/// Possible Barter-Data Stream types a [`Subscription`] is associated with.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamKind {
    Trades,
    Candles(Interval),
    Klines(Interval),
    OrderBookDeltas,
    OrderBooks,
}

impl Display for StreamKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StreamKind::Trades => "trades".to_owned(),
            StreamKind::Candles(interval) => format!("candles_{}", interval),
            StreamKind::Klines(interval) => format!("klines_{}", interval),
            StreamKind::OrderBookDeltas => "order_book_deltas".to_owned(),
            StreamKind::OrderBooks => "order_books".to_owned()
        })

    }
}

/// Barter new type representing a time interval `String` identifier.
///
/// eg/ "1m", "1h", "12h", "1d", "1w", "1M", etc
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Interval(pub String);

impl Debug for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Interval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Interval {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Interval::new)
    }
}

impl<S> From<S> for Interval
where
    S: Into<String>
{
    fn from(input: S) -> Self {
        Self(input.into())
    }
}

impl Interval {
    /// Construct an [`Interval`] new type using the provided `Into<Interval>` value.
    pub fn new<S>(input: S) -> Self
        where
            S: Into<Interval>
    {
        input.into()
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Metadata generated from a collection of Barter [`Subscription`]s. This includes the exchange
/// specific subscription payloads that are sent to the exchange.
pub struct SubscriptionMeta {
    /// `HashMap` containing the mapping between an incoming exchange message's [`SubscriptionId`],
    /// and a Barter [`Subscription`]. Used to identify the original [`Subscription`] associated
    /// with a received message.
    pub ids: SubscriptionIds,
    /// Number of [`Subscription`] responses expected from the exchange. Used to validate all
    /// [`Subscription`] were accepted.
    pub expected_responses: usize,
    /// Collection of [`WsMessage`]s containing exchange specific subscription payloads to be sent.
    pub subscriptions: Vec<WsMessage>,
}

/// Convenient type alias for a `HashMap` containing the mapping between an incoming exchange
/// message's [`SubscriptionId`], and a Barter [`Subscription`]. Used to identify the original
/// [`Subscription`] associated with a received message.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct SubscriptionIds(pub HashMap<SubscriptionId, Subscription>);

impl SubscriptionIds {
    /// Find the [`Instrument`] associated with the provided `Into<SubscriptionId>`.
    pub fn find_instrument<Id>(&self, id: Id) -> Result<Instrument, SocketError>
    where
        Id: Into<SubscriptionId>
    {
        let subscription_id: SubscriptionId = id.into();
        self.0
            .get(&subscription_id)
            .map(|subscription| subscription.instrument.clone())
            .ok_or(SocketError::Unidentifiable(subscription_id))
    }
}

/// New type representing a unique `String` identifier for a stream that has been subscribed to.
/// This identifier is used to associated a [`Subscription`] with data structures received from
/// the exchange.
///
/// Note: Each exchange will require the use of different `String` identifiers depending on the
/// data structures they send.
///
/// eg/ [`SubscriptionId`] of an `FtxTrade` is "{BASE}/{QUOTE}" (ie/ market).
/// eg/ [`SubscriptionId`] of a `BinanceTrade` is "{base}{symbol}@trade" (ie/ channel).
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct SubscriptionId(pub String);

impl Debug for SubscriptionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SubscriptionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SubscriptionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SubscriptionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(SubscriptionId)
    }
}

impl<S> From<S> for SubscriptionId
where
    S: Into<String>,
{
    fn from(input: S) -> Self {
        Self(input.into())
    }
}

/// Barter new type representing a monotonically increasing `u64` sequence number. Used for tracking
/// the order of received messages via an `ExchangeSocket`.
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