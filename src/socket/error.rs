use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("error subscribing to resources over the socket: {0}")]
    SubscribeError(String),

    #[error("received unidentifiable message over the socket: {0}")]
    Unidentifiable(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON SerDe error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Sink error")]
    SinkError,
}