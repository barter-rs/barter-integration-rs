use thiserror::Error;

/// All integration related errors generated in `barter-integration`.
#[derive(Debug, Error)]
pub enum SocketError {
    #[error("SerDe JSON error: {error} when deserialising payload: {payload}")]
    Serde {
        error: serde_json::Error,
        payload: String,
    },

    #[error("Sink error")]
    Sink,

    #[error("ExchangeSocket terminated with closing frame: {0}")]
    Terminated(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
}