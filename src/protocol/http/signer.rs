use crate::{
    SocketError,
    protocol::http::rest::RestRequest
};
use reqwest::RequestBuilder;

/// Generates and encodes a Http request signature.
pub trait SignatureGenerator {
    type Encoder: Encoder;

    /// Generates a [`RestRequest`] signature and encodes it into the required `String` format.
    fn signature<Request>(&self, request: &Request, builder: &RequestBuilder) -> Result<String, SocketError>
    where
        Request: RestRequest,
    {
        self.to_signature_bytes::<Request>(request, builder)
            .map(Self::Encoder::encode)
    }

    /// Generates the signature associated with the provided [`RestRequest`]. This will contain
    /// API specific logic.
    ///
    /// # Examples
    ///
    /// ## Ftx: Private Http GET Request
    /// ```rust,ignore
    /// fn to_signature_bytes<Request>(&self, request: &Request, builder: &RequestBuilder) -> Result<&[u8], SocketError>
    /// where
    ///     Request: RestRequest
    /// {
    ///     // Current millisecond timestamp
    ///     let time = Utc::now().timestamp_millis().to_string();
    ///
    ///     // Generate bytes to sign
    ///     format!("{time}{}{}", Request::method(), Request::path()).as_bytes()
    /// }
    /// ```
    fn to_signature_bytes<Request>(&self, request: &Request, builder: &RequestBuilder) -> Result<&[u8], SocketError>
    where
        Request: RestRequest;
}

/// Encodes bytes data.
pub trait Encoder {
    /// Encodes the bytes data into some `String` format.
    fn encode<Bytes>(data: Bytes) -> String
    where
        Bytes: AsRef<[u8]>;
}

/// Signs Http requests and adds any required headers.
pub trait Signer {
    /// Adds an authorisation signature to the [`RequestBuilder`], any required headers, and then
    /// builds the [`reqwest::Request`]. This will contain API specific logic.
    ///
    /// # Examples
    ///
    /// ## Ftx: Private Http GET Request
    /// ```rust,ignore
    /// fn sign_request(&self, builder: RequestBuilder, signature: String) -> Result<reqwest::Request, SocketError> {
    ///     // Add Ftx required Headers & build reqwest::Request
    ///     builder
    ///         .header(HEADER_FTX_KEY, &self.api_key)
    ///         .header(HEADER_FTX_SIGN, &signature)
    ///         .header(HEADER_FTX_TS, &time)
    ///         .build()
    ///         .map_err(SocketError::from)
    /// }
    /// ```
    fn sign_request(&self, builder: RequestBuilder, signature: String) -> Result<reqwest::Request, SocketError>;
}

/// Responsible for generating [`RestRequest`] signatures and completing [`RequestBuilder`] by
/// adding the encoded signature to the correct [`reqwest::Request`] location (header, body, etc).
#[derive(Debug)]
pub struct SignManager<SigGen, Sig>
where
    SigGen: SignatureGenerator,
    Sig: Signer,
{
    pub generator: SigGen,
    pub signer: Sig,
}

/// Encodes bytes data as a hex `String` using lowercase characters.
#[derive(Debug, Copy, Clone)]
pub struct HexEncoder;

impl Encoder for HexEncoder {
    fn encode<Bytes>(data: Bytes) -> String where Bytes: AsRef<[u8]> {
        hex::encode(data)
    }
}
