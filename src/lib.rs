#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
)]

///! # Barter-Integration

use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};
use serde::{Deserialize, Deserializer, Serialize};

/// Todo:
pub mod socket;
pub mod util;

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

/// Barter new type representing a monotonically increasing `u64` sequence number. Used for tracking
/// the order of received messages via an [`ExchangeSocket`].
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