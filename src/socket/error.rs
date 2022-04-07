use thiserror::Error;

/// All integration related errors generated in `barter-integration`.
#[derive(Debug, Error)]
pub enum SocketError {
    #[error("error subscribing to resources over the socket: {0}")]
    Subscribe(String),

    #[error("Sink error")]
    Sink,

    #[error("consumed unidentifiable message: {0}")]
    Unidentifiable(String),

    #[error("SerDe JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("ExchangeSocket terminated with closing frame: {0}")]
    Terminated(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
}