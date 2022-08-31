use crate::model::SubscriptionId;
use thiserror::Error;

/// All integration related errors generated in `barter-integration`.
#[derive(Debug, Error)]
pub enum SocketError {
    #[error("Sink error")]
    Sink,

    #[error("SerDe JSON error: {error} when deserialising payload: {payload}")]
    Serde {
        error: serde_json::Error,
        payload: String,
    },

    #[error("error subscribing to resources over the socket: {0}")]
    Subscribe(String),

    #[error("ExchangeStream terminated with closing frame: {0}")]
    Terminated(String),

    #[error("{entity} does not support: {item}")]
    Unsupported { entity: &'static str, item: String },

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("consumed unidentifiable message: {0}")]
    Unidentifiable(SubscriptionId),
}
