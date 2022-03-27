use std::fmt::{Display, Formatter};
use crate::{
    socket::{
        ExchangeSocket, Transformer,
        protocol::websocket::{connect, WebSocketParser, WsMessage, ExchangeWebSocket},
        error::SocketError
    },
    public::model::{Subscription, StreamId, MarketEvent},
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use futures::{SinkExt, Stream};

/// Todo:
pub mod model;
pub mod binance;
pub mod explore;

// Todo: Rust docs, add basic impls, change name to ExchangeId ? Produce &'static str
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum ExchangeId {
    BinanceFutures,
    Binance,
    Ftx,
}

impl Display for ExchangeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ExchangeId::BinanceFutures => "binance_futures",
            ExchangeId::Binance => "binance",
            ExchangeId::Ftx => "ftx",
        })
    }
}

/// Todo:
pub trait StreamIdentifier {
    fn to_stream_id(&self) -> StreamId;
}

/// Todo:
#[async_trait]
pub trait MarketStream<OutputIter>: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin
where
    OutputIter: IntoIterator<Item = MarketEvent>,
{
    async fn init(subscriptions: &[Subscription]) -> Result<Self, SocketError>;
}

/// Todo:
pub trait ExchangeTransformer: Sized
where
    Self: Transformer<MarketEvent>,
{
    const EXCHANGE: ExchangeId;
    const BASE_URL: &'static str;
    fn new() -> Self;
    fn generate_subscriptions(&mut self, subscriptions: &[Subscription]) -> Vec<serde_json::Value>;
}

#[async_trait]
impl<ExchangeT, OutputIter> MarketStream<OutputIter>
    for ExchangeWebSocket<ExchangeT, ExchangeT::Input, MarketEvent>
where
    Self: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin,
    ExchangeT: ExchangeTransformer + Send,
    OutputIter: IntoIterator<Item = MarketEvent>,
{
    async fn init(subscriptions: &[Subscription]) -> Result<Self, SocketError> {
        // Construct Exchange Transformer to translate between Barter & exchange data structures
        let mut exchange = ExchangeT::new();

        // Connect to exchange WebSocket server
        let mut websocket = connect(ExchangeT::BASE_URL).await?;

        // Action Subscriptions over the socket
        for sub_payload in exchange.generate_subscriptions(subscriptions) {
            websocket
                .send(WsMessage::Text(sub_payload.to_string()))
                .await?;
        }

        Ok(ExchangeSocket::new(websocket, WebSocketParser, exchange))
    }
}