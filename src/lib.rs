#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    // missing_docs
)]

///! # Barter-Integration

use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Deserializer, Serialize};

/// Todo:
pub mod socket;
pub mod util;

/// Todo:
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    pub kind: InstrumentKind,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("({}_{}, {}", self.base, self.quote, self.kind))
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
    /// Todo:
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
}

/// Todo:
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstrumentKind {
    Spot,
    Future(FutureKind),
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            InstrumentKind::Spot => "Spot".to_owned(),
            InstrumentKind::Future(kind) => format!("Future::{}", kind)
        })
    }
}

/// Defines the type of a `Future`. If the `Future` has metadata relating to it's expiry, this is
/// included.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FutureKind {
    Perpetual,
    Expiry,
}

impl Default for FutureKind {
    fn default() -> Self {
        Self::Perpetual
    }
}

impl Display for FutureKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FutureKind::Perpetual => "Perpetual",
            FutureKind::Expiry => "Expiry",
        })
    }
}

/// Barter new type representing a currency symbol `String` identifier.
///
/// eg/ "btc", "eth", "usdt", etc
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Symbol(pub String);

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

impl<S> From<S> for Symbol where S: Into<String> {
    fn from(input: S) -> Self {
        Self(input.into().to_lowercase())
    }
}

impl Symbol {
    /// Todo:
    pub fn new<S>(input: S) -> Self where S: Into<Symbol> {
        input.into()
    }
}

/// Todo:
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