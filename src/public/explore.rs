use crate::{
    public::{
        ExchangeId, MarketStream,
        model::{Subscription, MarketEvent},
        binance::futures::{BinanceFuturesItem, BinanceFuturesStream}
    },
    socket::error::SocketError, Symbol
};
use std::collections::HashMap;
use futures::StreamExt;
use rust_decimal::prelude::Zero;
use tokio::sync::mpsc;
use tracing::warn;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

// Todo:
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct StreamBuilder {
    subscriptions: HashMap<ExchangeId, Vec<Subscription>>,
}

// Todo: Probably make struct, impls, rust docs
pub struct Streams(pub HashMap<ExchangeId, UnboundedReceiver<MarketEvent>>);

impl Streams {
    pub fn select(&mut self, exchange: ExchangeId) -> UnboundedReceiver<MarketEvent> {
        self.0
            .remove(&exchange)
            .unwrap()
    }

    pub async fn join(mut self) -> UnboundedReceiverStream<MarketEvent>
}

impl StreamBuilder {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe<SubIter, Sub>(mut self, exchange: ExchangeId, subscriptions: SubIter) -> Self
    where
        SubIter: IntoIterator<Item = Sub>,
        Sub: Into<Subscription>,
    {
        self.subscriptions
            .insert(exchange, subscriptions.into_iter().map(Sub::into).collect());

        self
    }

    pub async fn init(self) -> Result<Streams, SocketError> {
        // Determine the number of Subscriptions
        let num_subs = self.subscriptions.len();
        if num_subs.is_zero() {
            return Err(SocketError::SubscribeError("no provided Subscriptions to action".to_owned()));
        }

        // Construct HashMap containing each Exchange's stream receiver
        let mut exchange_streams = HashMap::with_capacity(num_subs);

        for (exchange, subscriptions) in self.subscriptions {

            let exchange_rx = match exchange {
                ExchangeId::BinanceFutures => {
                    consume::<BinanceFuturesStream, BinanceFuturesItem>(&subscriptions).await?
                },
                not_supported => {
                    return Err(SocketError::SubscribeError(not_supported.to_string()))
                }
            };

            exchange_streams.insert(exchange, exchange_rx);
        }

        Ok(Streams(exchange_streams))
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

