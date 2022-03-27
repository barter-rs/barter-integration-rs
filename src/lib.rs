use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Deserializer, Serialize};

/// Todo:
pub mod socket;
pub mod public;
pub mod util;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    pub kind: InstrumentKind,
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum InstrumentKind {
    Spot,
    Future,
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    pub fn new<S>(input: S) -> Self where S: Into<Symbol> {
        input.into()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use futures::StreamExt;
    use crate::public::MarketStream;
    use crate::public::binance::futures::{BinanceFuturesItem, BinanceFuturesStream};
    use crate::public::explore::{Exchange, StreamBuilder, StreamKind};
    use crate::public::model::{MarketEvent, Subscription};
    use super::*;

    // Todo: Add subscription validation - it currently fails silently

    // Todo: Maybe OutputIter will become an Option<OutputIter>?

    // Todo: Add proper error enum for BinanceMessage in Barter-Data
    //     '--> eg/ enum BinanceMessage { Error, BinancePayload }

    // Todo: Do I want to keep the name trait Exchange? Do I like the generic ExTransformer, etc.


    #[tokio::test]
    async fn it_works() {
        let subscriptions = [
            Subscription::Trades(Instrument::new(
                "btc", "usdt", InstrumentKind::Future)
            ),
            Subscription::Trades(Instrument::new(
                "eth", "usdt", InstrumentKind::Future)
            ),
        ];

        run::<BinanceFuturesStream, BinanceFuturesItem>(&subscriptions).await;
    }

    #[tokio::test]
    async fn stream_builder_works() {
        let streams = [
            // ("btc", "usdt", InstrumentKind::Future, StreamKind::Trade),
            // ("eth", "usdt", InstrumentKind::Future, StreamKind::Trade),
            ("ltc", "usdt", InstrumentKind::Future, StreamKind::Trade),
        ];

        let mut binance_rx = StreamBuilder::new()
            .add(Exchange::BinanceFutures, streams)
            .build()
            .await.unwrap()
            .remove(&Exchange::BinanceFutures).unwrap();

        while let Some(event) = binance_rx.recv().await {
            println!("{:?}", event)
        }
    }
}