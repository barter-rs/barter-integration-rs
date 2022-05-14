use crate::socket::{SocketError, protocol::ProtocolParser};
use std::fmt::Debug;
use serde::{
    {Deserialize, Serialize},
    de::DeserializeOwned
};
use tokio_tungstenite::{
    connect_async, MaybeTlsStream,
    tungstenite::{
        client::IntoClientRequest,
        protocol::CloseFrame
    }
};
use tokio::net::TcpStream;
use tracing::{debug, trace, warn};

/// Convenient type alias for a tungstenite `WebSocketStream`.
pub type WebSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Communicative type alias for a tungstenite [`WebSocket`] `Message`.
pub type WsMessage = tokio_tungstenite::tungstenite::Message;

/// Communicative type alias for a tungstenite [`WebSocket`] `Error`.
pub type WsError = tokio_tungstenite::tungstenite::Error;

/// Default [`ProtocolParser`] implementation for a [`WebSocket`].
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct WebSocketParser;

impl ProtocolParser for WebSocketParser {
    type Message = WsMessage;
    type Error = WsError;

    fn parse<Output>(input: Result<Self::Message, Self::Error>) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned,
    {
        match input {
            Ok(ws_message) => match ws_message {
                WsMessage::Text(text) => process_text(text),
                WsMessage::Binary(binary) => process_binary(binary),
                WsMessage::Ping(ping) => process_ping(ping),
                WsMessage::Pong(pong) => process_pong(pong),
                WsMessage::Close(close_frame) => process_close_frame(close_frame),
            },
            Err(ws_err) => Some(Err(SocketError::WebSocket(ws_err)))
        }
    }
}

/// Process a payload of `String` by deserialising into an `ExchangeMessage`.
pub fn process_text<ExchangeMessage>(payload: String) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(serde_json::from_str::<ExchangeMessage>(&payload)
        .map_err(|error| {
            warn!(
                ?error,
                ?payload,
                action = "returning Some(Err(err))",
                "failed to deserialize WebSocket Message into domain specific Message"
            );
            SocketError::Serde { error, payload }
        })
    )
}

/// Process a payload of `Vec<u8>` bytes by deserialising into an `ExchangeMessage`.
pub fn process_binary<ExchangeMessage>(payload: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>>
where
    ExchangeMessage: DeserializeOwned,
{
    Some(serde_json::from_slice::<ExchangeMessage>(&payload)
        .map_err(|error| {
            warn!(
                ?error,
                ?payload,
                action = "returning Some(Err(err))",
                "failed to deserialize WebSocket Message into domain specific Message"
            );
            SocketError::Serde {
                error,
                payload: String::from_utf8(payload)
                    .unwrap_or_else(|x| x.to_string())
            }
        })
    )
}

/// Basic process for a WebSocket ping message. Logs the payload at `trace` level.
pub fn process_ping<ExchangeMessage>(ping: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>> {
    trace!(payload = ?ping, "received Ping WebSocket message");
    None
}

/// Basic process for a WebSocket pong message. Logs the payload at `trace` level.
pub fn process_pong<ExchangeMessage>(pong: Vec<u8>) -> Option<Result<ExchangeMessage, SocketError>> {
    trace!(payload = ?pong, "received Pong WebSocket message");
    None
}

/// Basic process for a WebSocket CloseFrame message. Logs the payload at `trace` level.
pub fn process_close_frame<ExchangeMessage>(close_frame: Option<CloseFrame<'_>>) -> Option<Result<ExchangeMessage, SocketError>> {
    let close_frame = format!("{:?}", close_frame);
    debug!(payload = %close_frame, "received CloseFrame WebSocket message");
    Some(Err(SocketError::Terminated(close_frame)))
}

/// Connect asynchronously to a [`WebSocket`] server.
pub async fn connect<R>(request: R) -> Result<WebSocket, SocketError>
where
    R: IntoClientRequest + Unpin + Debug
{
    debug!(?request, "attempting to establish WebSocket connection");
    connect_async(request)
        .await
        .map(|(websocket, _)| websocket)
        .map_err(SocketError::WebSocket)
}