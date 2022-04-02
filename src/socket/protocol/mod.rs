use crate::socket::{
    SocketError,
};
use serde::de::DeserializeOwned;

pub mod websocket;

/// Todo:
pub trait ProtocolParser<Output>
where
    Output: DeserializeOwned,
{
    type Input;
    fn parse(input: Self::Input) -> Option<Result<Output, SocketError>>;
}