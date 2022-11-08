use crate::SocketError;
use serde::de::DeserializeOwned;

/// Contains useful `WebSocket` type aliases and a default `WebSocket` implementation of a
/// [`ProtocolParser`].
pub mod websocket;

/// Contains HTTP client capable of executing signed & unsigned requests, as well as an associated
/// exchange oriented HTTP request.
pub mod http;

pub mod http_old;

/// `ProtocolParser`s are capable of parsing the input messages from a given protocol (eg WebSocket,
/// Financial Information eXchange, etc) and deserialising into an `Output`.
pub trait ProtocolParser {
    type Message;
    type Error;

    fn parse<Output>(input: Result<Self::Message, Self::Error>) -> Option<Result<Output, SocketError>>
    where
        Output: DeserializeOwned;
}
