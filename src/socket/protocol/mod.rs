use crate::socket::{
    SocketError,
};
use serde::de::DeserializeOwned;

pub mod websocket;

/// `ProtocolParser`s are capable of parsing the input messages from a given protocol (eg WebSocket,
/// Financial Information eXchange, etc) and deserialising into an `Output`.
pub trait ProtocolParser<Output>
where
    Output: DeserializeOwned,
{
    type Input;
    fn parse(input: Self::Input) -> Option<Result<Output, SocketError>>;
}