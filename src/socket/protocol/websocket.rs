use crate::socket::{SocketError, protocol::ProtocolParser, ExchangeSocket};
use std::fmt::Debug;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream,
    tungstenite::{
        client::IntoClientRequest,
        protocol::CloseFrame
    }
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tracing::{debug, trace, warn};

/// Convenient type alias for a tungstenite `WebSocketStream`.
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Communicative type alias for a tungstenite [`WebSocket`] `Message`.
pub type WsMessage = tokio_tungstenite::tungstenite::Message;

/// Communicative type alias for a tungstenite [`WebSocket`] `Error`.
pub type WsError = tokio_tungstenite::tungstenite::Error;

/// Convenient type alias for an [`ExchangeSocket`] utilising a tungstenite [`WebSocket`].
pub type ExchangeWebSocket<Exchange, ExMessage, Output> = ExchangeSocket<
    WebSocket, WsMessage, WebSocketParser, Exchange, ExMessage, Output>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct WebSocketParser;

impl<ExchangeMessage> ProtocolParser<ExchangeMessage> for WebSocketParser
where
    ExchangeMessage: DeserializeOwned,
{
    type Input = Result<WsMessage, WsError>;

    fn parse(input: Self::Input) -> Option<Result<ExchangeMessage, SocketError>> {
        match input {
            Ok(ws_message) => match ws_message {
                WsMessage::Text(text) => process_payload(text.into_bytes()),
                WsMessage::Binary(binary) => process_payload(binary),
                WsMessage::Ping(ping) => process_ping(ping),
                WsMessage::Pong(pong) => process_pong(pong),
                WsMessage::Close(close_frame) => process_close_frame(close_frame),
            },
            Err(ws_err) => Some(Err(SocketError::WebSocketError(ws_err)))
        }
    }
}

/// Process a payload of bytes by deserialising into an `ExchangeMessage`.
pub fn process_payload<ExchangeMessage>(payload: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(serde_json::from_slice::<ExchangeMessage>(&payload)
        .map_err(|err| {
            warn!(
                error = &*format!("{:?}", err),
                payload = &*format!("{:?}", payload),
                action = "returning Some(Err(err))",
                "failed to deserialize WebSocket Message into domain specific Message"
            );
            SocketError::SerdeJsonError(err)
        })
    )
}

/// Basic process for a WebSocket ping message. Logs the payload at `trace` level.
pub fn process_ping<ExchangeMessage>(ping: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>> {
    trace!(payload = &*format!("{:?}", ping), "received Ping WebSocket message");
    None
}

/// Basic process for a WebSocket pong message. Logs the payload at `trace` level.
pub fn process_pong<ExchangeMessage>(pong: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>> {
    trace!(payload = &*format!("{:?}", pong), "received Pong WebSocket message");
    None
}

/// Basic process for a WebSocket CloseFrame message. Logs the payload at `trace` level.
pub fn process_close_frame<ExchangeMessage>(close_frame: Option<CloseFrame<'_>>) -> Option<Result<ExchangeMessage, SocketError>> {
    let close_frame = format!("{:?}", close_frame);
    debug!(payload = &*close_frame, "received CloseFrame WebSocket message");
    Some(Err(SocketError::Terminated(close_frame)))
}

/// Connect asynchronously to [`WebSocket`] server.
pub async fn connect<R>(request: R) -> Result<WebSocket, SocketError>
where
    R: IntoClientRequest + Unpin + Debug
{
    debug!(request = &*format!("{:?}", request), "attempting to establish WebSocket connection");
    connect_async(request)
        .await
        .and_then(|(websocket, _)| Ok(websocket))
        .map_err(SocketError::WebSocketError)
}