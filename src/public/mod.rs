use crate::{
    socket::{
        ExchangeSocket, Transformer,
        protocol::websocket::{connect, WebSocket, WebSocketParser, WsMessage},
        error::SocketError
    },
    public::model::Subscription,
};
use async_trait::async_trait;
use futures::{SinkExt, Stream};
use serde::de::DeserializeOwned;
use crate::public::model::{MarketEvent, StreamId};

/// Todo:
pub mod model;
pub mod binance;

/// Todo:
pub trait StreamIdentifier {
    fn to_stream_id(&self) -> StreamId;
}

/// Todo:
#[async_trait]
pub trait MarketDataStream<OutputIter>: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin
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
impl<ExTransformer, ExMessage, OutputIter> MarketDataStream<OutputIter>
    for ExchangeSocket<WebSocket, WsMessage, WebSocketParser, ExTransformer, ExMessage, MarketEvent>
where
    Self: Stream<Item = Result<OutputIter, SocketError>> + Sized + Unpin,
    ExTransformer: Exchange<ExMessage> + Send,
    ExMessage: DeserializeOwned,
    OutputIter: IntoIterator<Item = MarketEvent>,
{
    async fn init(subscriptions: &[Subscription]) -> Result<Self, SocketError> {
        // Connect to exchange WebSocket server
        let mut websocket = connect(ExTransformer::BASE_URL).await?;

        // Construct Exchange capable of translating
        let mut exchange = ExTransformer::new();

        // Action Subscriptions over the socket
        for sub_payload in exchange.generate_subscriptions(subscriptions) {
            websocket
                .send(WsMessage::Text(sub_payload.to_string()))
                .await?;
        }

        Ok(ExchangeSocket::new(websocket, WebSocketParser, exchange))
    }
}