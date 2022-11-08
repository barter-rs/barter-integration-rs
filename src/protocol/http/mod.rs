use crate::error::SocketError;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use tracing::error;

pub mod rest;
pub mod signer;

pub trait HttpParser {
    type Error: From<SocketError>;

    /// Attempt to parse a [`StatusCode`] & payload `&[u8]` bytes into deserialisable `Response`.
    fn parse<Response>(&self, status: StatusCode, payload: &[u8]) -> Result<Response, Self::Error>
    where
        Response: DeserializeOwned,
    {
        // Attempt to deserialise reqwest::Response bytes into Ok(Response)
        let parse_ok_error = match serde_json::from_slice::<Response>(&payload) {
            Ok(response) => return Ok(response),
            Err(serde_error) => serde_error,
        };

        // Attempt to deserialise API ExchangeError if Ok(Response) deserialisation failed
        let parse_error_error = match self.parse_api_error(status, &payload) {
            Ok(api_error) => return Err(api_error),
            Err(serde_error) => serde_error,
        };

        // Log errors if failed to deserialise reqwest::Response into Response or API Self::Error
        error!(
            status_code = ?status,
            ?parse_ok_error,
            ?parse_error_error,
            response_body = %String::from_utf8_lossy(&payload),
            "error deserializing HTTP response"
        );

        Err(Self::Error::from(SocketError::DeserialiseBinary {
            error: parse_ok_error,
            payload: payload.to_vec(),
        }))
    }

    /// If [`parse`](Self::parse) fails to deserialise the `Ok(Response)`, this function attempts
    /// to parse the API [`Self::Error`] associated with the response.
    fn parse_api_error(&self, status: StatusCode, payload: &[u8]) -> Result<Self::Error, SocketError>;
}