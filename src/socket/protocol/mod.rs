use crate::socket::{
    SocketError,
};
use serde::de::DeserializeOwned;

/// Contains useful `WebSocket` type aliases and a default `WebSocket` implementation of a
/// [`ProtocolParser`].
pub mod websocket;

/// `ProtocolParser`s are capable of parsing the input messages from a given protocol (eg WebSocket,
/// Financial Information eXchange, etc) and deserialising into an `Output`.
pub trait ProtocolParser {
    type ProtocolMessage;

    fn parse<Output>(input: Self::ProtocolMessage) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned;
}