use crate::{Instrument, InstrumentKind, public::{
    MarketStream,
    model::{Subscription, MarketEvent},
    binance::futures::{BinanceFuturesItem, BinanceFuturesStream},
}, socket::error::SocketError, Symbol};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use futures::StreamExt;
use rust_decimal::prelude::Zero;
use tokio::sync::mpsc;
use tracing::warn;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Exchange {
    BinanceFutures,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct StreamConfig {
    pub instrument: Instrument,
    pub kind: StreamKind,
}

impl<I> From<(I, StreamKind)> for StreamConfig
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

impl<S> From<(S, S, InstrumentKind, StreamKind)> for StreamConfig
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum StreamKind {
    Trade,
}


pub struct StreamBuilder {
    config: HashMap<Exchange, Vec<StreamConfig>>,
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    pub fn add<ConfigIter, Config>(mut self, exchange: Exchange, config: ConfigIter) -> Self
    where
        ConfigIter: IntoIterator<Item = Config>,
        Config: Into<StreamConfig>,
    {
        self.config.insert(exchange, config.into_iter().map(Config::into).collect());
        self
    }

    pub async fn build(self) -> Result<HashMap<Exchange, mpsc::UnboundedReceiver<MarketEvent>>, SocketError> {
        if self.config.len().is_zero() {
            return Err(SocketError::SubscribeError("no provided streams to subscribe to"));
        }

        // Construct HashMap containing all each Exchange's stream receiver
        let mut exchange_streams = HashMap::with_capacity(self.config.len());

        for (exchange, streams) in self.config.into_iter() {

            let subscriptions = streams
                .into_iter()
                .map(Subscription::from)
                .collect::<Vec<Subscription>>();

            let exchange_rx = match exchange {
                Exchange::BinanceFutures => {
                    consume::<BinanceFuturesStream, BinanceFuturesItem>(&subscriptions).await?
                }
            };

            exchange_streams.insert(exchange, exchange_rx);
        }

        Ok(exchange_streams)
    }
}

async fn consume<S, OutputIter>(subscriptions: &[Subscription]) -> Result<mpsc::UnboundedReceiver<MarketEvent>, SocketError>
where
    S: MarketStream<OutputIter> + Send + 'static,
    OutputIter: IntoIterator<Item = MarketEvent>,
{
    let mut stream = S::init(subscriptions).await?;

    let (stream_tx, stream_rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {

            match result {
                Ok(events) => {
                    events
                        .into_iter()
                        .for_each(|event| {
                            let _ = stream_tx.send(event);
                        })
                }
                Err(err) => {
                    warn!(error = &*format!("{:?}", err), "received an stream error");
                    continue;
                }
            }
        }
    });

    Ok(stream_rx)
}

