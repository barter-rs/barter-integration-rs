use barter_integration::{
    protocol::http::{
        HttpParser, private::{Signer, RequestSigner, encoder::HexEncoder},
        rest::{RestRequest, client::RestClient},
    },
    error::SocketError, metric::Tag, model::Symbol,
};
use serde::Deserialize;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use hmac::{
    digest::KeyInit,
    Hmac,
};
use reqwest::{RequestBuilder, StatusCode};
use tokio::sync::mpsc;
use thiserror::Error;

struct FtxSigner { api_key: String, }

// Configuration required to sign every Ftx `RestRequest`
struct FtxSignConfig {
    api_key: String,
    time: DateTime<Utc>,
    method: reqwest::Method,
    path: &'static str,
}

impl Signer for FtxSigner {
    type Config = FtxSignConfig;

    fn config<Request>(&self, _: Request, _: &RequestBuilder) -> Self::Config
        where
            Request: RestRequest
    {
        FtxSignConfig {
            api_key: self.api_key.clone(),
            time: Utc::now(),
            method: Request::method(),
            path: Request::path()
        }
    }

    fn bytes_to_sign(config: &Self::Config) -> Result<Bytes, SocketError> {
        Ok(Bytes::from(format!("{}{}{}", config.time, config.method, config.path)))
    }

    fn build_signed_request(config: Self::Config, builder: RequestBuilder, signature: String) -> Result<reqwest::Request, SocketError> {
        // Add Ftx required Headers & build reqwest::Request
        builder
            .header("FTX-KEY", config.api_key)
            .header("FTX-TS", &config.time.timestamp_millis().to_string())
            .header("FTX-SIGN", &signature)
            .build()
            .map_err(SocketError::from)
    }
}

struct FtxParser;

impl HttpParser for FtxParser {
    type Error = ExecutionError;

    fn parse_api_error(&self, status: StatusCode, payload: &[u8]) -> Result<Self::Error, SocketError> {
        // Deserialise Ftx API error
        let error = serde_json::from_slice::<serde_json::Value>(payload)
            .map(|response| response.to_string())
            .map_err(|error| SocketError::DeserialiseBinary { error, payload: payload.to_vec()})?;

        // Parse Ftx error message to determine custom ExecutionError variant
        Ok(match error.as_str() {
            message if message.contains("Invalid login credentials") => {
                ExecutionError::Unauthorised(error)
            },
            _ => {
                ExecutionError::Socket(SocketError::HttpResponse(status, error))
            }
        })
    }
}

#[derive(Debug, Error)]
enum ExecutionError {
    #[error("request authorisation invalid: {0}")]
    Unauthorised(String),

    #[error("SocketError: {0}")]
    Socket(#[from] SocketError)
}

struct FetchBalancesRequest;

impl RestRequest for FetchBalancesRequest {
    type QueryParams = ();                  // FetchBalances does not require any QueryParams
    type Body = ();                         // FetchBalances does not require any Body
    type Response = FetchBalancesResponse;  // Define Response type

    fn path() -> &'static str {
        "/api/wallet/balances"
    }

    fn method() -> reqwest::Method {
        reqwest::Method::GET
    }

    fn metric_tag() -> Tag {
        Tag::new("method", "fetch_balances")
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FetchBalancesResponse {
    success: bool,
    result: Vec<FtxBalance>
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct FtxBalance {
    #[serde(rename = "coin")]
    symbol: Symbol,
    total: f64,
}

/// See Barter-Execution for a comprehensive real-life example, as well as code you can use out of the
/// box to execute trades on many exchanges.
#[tokio::main]
async fn main() {
    // Construct Metric channel to send Http execution metrics over
    let (http_metric_tx, _http_metric_rx) = mpsc::unbounded_channel();

    // HMAC-SHA256 encoded account API secret used for signing private http requests
    let mac: Hmac<sha2::Sha256> = Hmac::new_from_slice("api_secret".as_bytes()).unwrap();

    // Build Ftx configured RequestSigner for signing http requests with hex encoding
    let request_singer = RequestSigner::new(
        FtxSigner { api_key: "api_key".to_string()},
        mac,
        HexEncoder
    );

    // Build RestClient with Ftx configuration
    let rest_client = RestClient::new(
        "https://ftx.com",
        http_metric_tx,
        request_singer,
        FtxParser
    );

    // Fetch Result<FetchBalancesResponse, ExecutionError>
    let _response = rest_client
        .execute(FetchBalancesRequest)
        .await;
}