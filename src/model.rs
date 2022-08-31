use serde::{Deserialize, Deserializer, Serialize};
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
};

/// Represents a unique combination of an [`Exchange`] & an [`Instrument`].
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Market {
    pub exchange: Exchange,
    #[serde(flatten)]
    pub instrument: Instrument,
}

impl<E, I> From<(E, I)> for Market
where
    E: Into<Exchange>,
    I: Into<Instrument>,
{
    fn from((exchange, instrument): (E, I)) -> Self {
        Self::new(exchange, instrument)
    }
}

impl<E, S> From<(E, S, S, InstrumentKind)> for Market
where
    E: Into<Exchange>,
    S: Into<Symbol>,
{
    fn from((exchange, base, quote, instrument_kind): (E, S, S, InstrumentKind)) -> Self {
        Self::new(exchange, (base, quote, instrument_kind))
    }
}

impl Market {
    /// Constructs a new [`Market`] using the provided [`Exchange`] & [`Instrument`].
    pub fn new<E, I>(exchange: E, instrument: I) -> Self
    where
        E: Into<Exchange>,
        I: Into<Instrument>,
    {
        Self {
            exchange: exchange.into(),
            instrument: instrument.into(),
        }
    }
}

/// Barter new type representing a unique `String` identifier for a [`Market`], where a [`Market`]
/// represents an [`Instrument`] being traded on an [`Exchange`].
///
/// eg/ binance_btc_usdt_spot
/// eg/ ftx_btc_usdt_future_perpetual
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct MarketId(pub String);

impl<'a, M> From<M> for MarketId
where
    M: Into<&'a Market>,
{
    fn from(market: M) -> Self {
        let market = market.into();
        Self::new(&market.exchange, &market.instrument)
    }
}

impl Debug for MarketId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for MarketId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for MarketId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(MarketId)
    }
}

impl MarketId {
    /// Construct a unique `String` [`MarketId`] identifier for a [`Market`], where a [`Market`]
    /// represents an [`Instrument`] being traded on an [`Exchange`].
    pub fn new(exchange: &Exchange, instrument: &Instrument) -> Self {
        Self(
            format!(
                "{}_{}_{}_{}",
                exchange, instrument.base, instrument.quote, instrument.kind
            )
            .to_lowercase(),
        )
    }
}

/// Barter representation of an [`Exchange`]'s name.
///
/// eg/ Exchange("binance"), Exchange("bitfinex"), Exchange("ftx"), etc.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Exchange(Cow<'static, str>);

impl<E> From<E> for Exchange
where
    E: Into<Cow<'static, str>>,
{
    fn from(exchange: E) -> Self {
        Exchange(exchange.into())
    }
}

impl Debug for Exchange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Exchange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Barter representation of an `Instrument`. Used to uniquely identify a `base_quote` pair, and it's
/// associated instrument type.
///
/// eg/ Instrument { base: "btc", quote: "usdt", kind: Spot }
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    #[serde(rename = "instrument_type")]
    pub kind: InstrumentKind,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}_{}, {})", self.base, self.quote, self.kind)
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
            kind,
        }
    }
}

impl Instrument {
    /// Constructs a new [`Instrument`] using the provided configuration.
    pub fn new<S>(base: S, quote: S, kind: InstrumentKind) -> Self
    where
        S: Into<Symbol>,
    {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind,
        }
    }
}

/// Defines the type of [`Instrument`] which is being traded on a given `base_quote` market.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentKind {
    Spot,
    FuturePerpetual,
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InstrumentKind::Spot => "spot",
                InstrumentKind::FuturePerpetual => "future_perpetual",
            }
        )
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
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Symbol::new)
    }
}

impl<S> From<S> for Symbol
where
    S: Into<String>,
{
    fn from(input: S) -> Self {
        Symbol::new(input)
    }
}

impl Symbol {
    /// Construct a new [`Symbol`] new type using the provided `Into<Symbol>` value.
    pub fn new<S>(input: S) -> Self
    where
        S: Into<String>,
    {
        Self(input.into().to_lowercase())
    }
}

/// New type representing a unique `String` identifier for a stream that has been subscribed to.
/// This is used to identify data structures received over the socket.
///
/// For example, `Barter-Data` uses this identifier to associate received data structures from the
/// exchange with the original `Barter-Data` `Subscription` that was actioned over the socket.
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

/// [`Side`] of a trade - Buy or Sell.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Side {
    #[serde(alias = "buy", alias = "BUY")]
    Buy,
    #[serde(alias = "sell", alias = "SELL")]
    Sell,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::Error;

    #[test]
    fn test_deserialise_market() {
        struct TestCase {
            input: &'static str,
            expected: Result<Market, serde_json::Error>,
        }

        let cases = vec![
            TestCase {
                // TC0: Valid Binance btc_usd Spot Market
                input: r##"{ "exchange": "binance", "base": "btc", "quote": "usd", "instrument_type": "spot" }"##,
                expected: Ok(Market {
                    exchange: Exchange::from("binance"),
                    instrument: Instrument::from(("btc", "usd", InstrumentKind::Spot)),
                }),
            },
            TestCase {
                // TC1: Valid Ftx btc_usd FuturePerpetual Market
                input: r##"{ "exchange": "ftx", "base": "btc", "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Ok(Market {
                    exchange: Exchange::from("ftx"),
                    instrument: Instrument::from(("btc", "usd", InstrumentKind::FuturePerpetual)),
                }),
            },
            TestCase {
                // TC2: Invalid Market w/ numeric exchange
                input: r##"{ "exchange": 100, "base": "btc", "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Err(serde_json::Error::custom("")),
            },
            TestCase {
                // TC3: Invalid Market w/ gibberish
                input: r##"{ "gibberish": "shouldfail"}"##,
                expected: Err(serde_json::Error::custom("")),
            },
        ];

        for (index, test) in cases.into_iter().enumerate() {
            let actual = serde_json::from_str::<Market>(test.input);

            match (actual, test.expected) {
                (Ok(actual), Ok(expected)) => {
                    assert_eq!(actual, expected, "TC{} failed", index)
                }
                (Err(_), Err(_)) => {
                    // Test passed
                }
                (actual, expected) => {
                    // Test failed
                    panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                }
            }
        }
    }

    #[test]
    fn test_deserialise_instrument() {
        struct TestCase {
            input: &'static str,
            expected: Result<Instrument, serde_json::Error>,
        }

        let cases = vec![
            TestCase {
                // TC0: Valid Spot Instrument
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "spot" }"##,
                expected: Ok(Instrument::from(("btc", "usd", InstrumentKind::Spot))),
            },
            TestCase {
                // TC1: Valid FuturePerpetual Instrument
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Ok(Instrument::from((
                    "btc",
                    "usd",
                    InstrumentKind::FuturePerpetual,
                ))),
            },
            TestCase {
                // TC2: Invalid Spot Instrument w/ numeric base
                input: r##"{ "base": 100, "quote": "usd", "instrument_type": "future_perpetual" }"##,
                expected: Err(serde_json::Error::custom("")),
            },
            TestCase {
                // TC3: Invalid Instrument w/ gibberish InstrumentKind
                input: r##"{"base": "btc", "quote": "usd", "instrument_type": "gibberish" }"##,
                expected: Err(serde_json::Error::custom("")),
            },
            TestCase {
                // TC4: Invalid Instrument w/ complete gibberish
                input: r##"{ "gibberish": "shouldfail"}"##,
                expected: Err(serde_json::Error::custom("")),
            },
        ];

        for (index, test) in cases.into_iter().enumerate() {
            let actual = serde_json::from_str::<Instrument>(test.input);

            match (actual, test.expected) {
                (Ok(actual), Ok(expected)) => {
                    assert_eq!(actual, expected, "TC{} failed", index)
                }
                (Err(_), Err(_)) => {
                    // Test passed
                }
                (actual, expected) => {
                    // Test failed
                    panic!("TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n");
                }
            }
        }
    }
}
