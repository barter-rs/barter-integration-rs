use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("error subscribing to resources over the socket: {0}")]
    SubscribeError(String),

    #[error("ExchangeSocket terminated with closing frame: {0}")]
    Terminated(String),

    #[error("consumed unidentifiable message: {0}")]
    Unidentifiable(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON SerDe error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Sink error")]
    SinkError,
}