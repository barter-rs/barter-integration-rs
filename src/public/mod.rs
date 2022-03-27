use crate::{
    socket::{
        ExchangeSocket, Transformer,
        protocol::websocket::{connect, WebSocket, WebSocketParser, WsMessage, ExchangeWebSocket},
        error::SocketError
    },
    public::model::{Subscription, StreamId, MarketEvent},
};
use async_trait::async_trait;
use futures::{Sink, SinkExt, Stream};
use serde::de::DeserializeOwned;

/// Todo:
pub mod model;
pub mod binance;
pub mod explore;

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
pub trait Exchange<ExMessage>: Sized
where
    Self: Transformer<ExMessage, MarketEvent>,
    ExMessage: DeserializeOwned,
{
    const EXCHANGE: &'static str;
    const BASE_URL: &'static str;
    fn new() -> Self;
    fn generate_subscriptions(&mut self, subscriptions: &[Subscription]) -> Vec<serde_json::Value>;
}

#[async_trait]
impl<ExchangeT, ExMessage, OutputIter> MarketStream<OutputIter>
    for ExchangeWebSocket<ExchangeT, ExMessage, OutputIter>
where
    Self: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin,
    ExchangeT: Exchange<ExMessage> + Send,
    ExMessage: DeserializeOwned,
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