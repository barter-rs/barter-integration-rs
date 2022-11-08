use crate::SocketError;
use reqwest::{Request, RequestBuilder};
use hmac::{Hmac, Mac};
use crate::protocol::http::rest::RestRequest;

pub trait Signer {
    fn sign<Request>(&self, request: &Request, builder: RequestBuilder) -> Result<reqwest::Request, SocketError>;
}

pub trait SignerNew {
    type Encoder: Encoder;

    fn sign<Request>(&self, request: &Request, builder: &RequestBuilder) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest;

    fn generate_signature<Bytes>(&self, request: &Request, builder: &RequestBuilder) -> Result<String, SocketError>
    where
        Bytes: AsRef<[u8]>
    {
        self.to_signature_bytes(request, builder)
            .map(Self::Encoder::encode)
    }

    fn to_signature_bytes<Request, Bytes>(&self, request: &Request, builder: &RequestBuilder) -> Result<Bytes, SocketError>
    where
        Request: RestRequest,
        Bytes: AsRef<[u8]>;
}

trait Encoder {
    fn encode<Bytes>(data: Bytes) -> String
    where
        Bytes: AsRef<[u8]>;
}

struct NoAuth;

impl Signer for NoAuth {
    fn sign<Request>(&self, _: &Request, builder: RequestBuilder) -> Result<reqwest::Request, SocketError> {
        builder
            .build()
            .map_err(SocketError::from)
    }
}

struct Hmac256<Encoder> {
    api_key: String,
    hmac: Hmac<sha2::Sha256>,
    encoder: Encoder,
}

