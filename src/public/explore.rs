use crate::{
    Instrument,
    public::{
        MarketStream,
        model::{Subscription, MarketEvent},
        binance::futures::{BinanceFuturesItem, BinanceFuturesStream},
    },
    socket::error::SocketError
};
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

    pub fn add(mut self, exchange: Exchange, config: Vec<StreamConfig>) -> Self {
        self.config.insert(exchange, config);
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
                .map(|stream| match stream.kind {
                    StreamKind::Trade => Subscription::Trades(stream.instrument)
                })
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

