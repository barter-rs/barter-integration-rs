use crate::{
    metric::{Field, Metric, Tag},
    SocketError
};
use super::{
    HttpParser,
    signer::{SignatureGenerator, Signer, SignManager},
};
use std::time::Duration;
use bytes::Bytes;
use chrono::Utc;
use reqwest::StatusCode;
use serde::{
    de::DeserializeOwned,
    Serialize,
};
use tokio::sync::mpsc;
use tracing::warn;

/// Default Http [`reqwest::Request`] timeout Duration.
const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Http REST request that can be executed by a [`RestClient`].
pub trait RestRequest {
    /// Serialisable query parameters type - use unit struct () if not required for this request.
    type QueryParams: Serialize;

    /// Serialisable Body type - use unit struct () if not required for this request.
    type Body: Serialize;

    /// Expected response type if this request was successful.
    type Response: DeserializeOwned;

    /// Additional [`Url`] path to the resource.
    fn path() -> &'static str;

    /// Http [`reqwest::Method`] of this request.
    fn method() -> reqwest::Method;

    /// [`Metric`] [`Tag`] that identifies this request.
    fn metric_tag() -> Tag;

    /// Generates the request [`reqwest::Url`] given the provided base API url.
    fn url<S: Into<String>>(&self, base_url: S) -> Result<reqwest::Url, SocketError>
    where
        S: Into<String>,
    {
        // Generate Url String
        let mut url = base_url.into() + Self::path();

        // Add optional query parameters
        if let Some(parameters) = self.query_params() {
            let query_string = serde_qs::to_string(parameters)?;
            url.push('?');
            url.push_str(&query_string);
        }

        reqwest::Url::parse(&url).map_err(SocketError::from)
    }

    /// Optional query parameters for this request.
    fn query_params(&self) -> Option<&Self::QueryParams> {
        None
    }

    /// Optional Body for this request.
    fn body(&self) -> Option<&Self::Body> {
        None
    }

    /// Http request timeout [`Duration`].
    fn timeout() -> Duration {
        DEFAULT_HTTP_REQUEST_TIMEOUT
    }
}

///
#[derive(Debug)]
pub struct RestClient<'a, SigGen, Sig, Parser>
where
    SigGen: SignatureGenerator,
    Sig: Signer,
{
    base_url: &'a str,
    http_client: reqwest::Client,
    metric_tx: mpsc::UnboundedSender<Metric>,
    sign: SignManager<SigGen, Sig>,
    parser: Parser,
}

impl<'a, SigGen, Sig, Parser> RestClient<'a, SigGen, Sig, Parser>
where
    SigGen: SignatureGenerator,
    Sig: Signer,
    Parser: HttpParser,
{

    /// Execute the provided [`RestRequest`].
    async fn execute<Request>(&self, request: Request) -> Result<Request::Response, Parser::Error>
    where
        Request: RestRequest,
    {
        // Use provided Request to construct a signed reqwest::Request
        let request = self.build(&request)?;

        // Measure request execution
        let (status, payload) = self
            .measured_execution::<Request>(request)
            .await?;

        // Attempt to parse API Success or Error response
        self.parser
            .parse::<Request::Response>(status, &payload)
    }

    /// Use the provided [`RestRequest`] to construct a signed Http [`reqwest::Request`].
    fn build<Request>(&self, request: &Request) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest
    {
        // Generate Url
        let url = request.url(self.base_url)?;

        // Construct RequestBuilder with method & url
        let mut builder = self
            .http_client
            .request(Request::method(), url)
            .timeout(Request::timeout());

        // Add optional request Body
        if let Some(body) = request.body() {
            builder = builder.json(body);
        }

        // Generate request signature
        let signature = self.sign.generator.signature::<Request>(request, &builder)?;

        // Sign reqwest::RequestBuilder with exchange specific method
        let request = self.sign.signer.sign_request(builder, signature)?;

        Ok(request)
    }

    /// Execute the built [`reqwest::Request`] using the [`reqwest::Client`].
    ///
    /// Measures the Http request round trip duration and sends the associated [`Metric`]
    /// via the [`Metric`] transmitter.
    async fn measured_execution<Request>(&self, request: reqwest::Request) -> Result<(StatusCode, Bytes), SocketError>
    where
        Request: RestRequest
    {
        // Measure the HTTP request round trip duration
        let start = std::time::Instant::now();
        let response = self.http_client.execute(request).await?;
        let duration = start.elapsed().as_millis() as u64;

        // Construct HTTP request duration Metric & send
        let http_duration = Metric {
            name: "http_request_duration",
            time: Utc::now().timestamp_millis() as u64,
            tags: vec![
                Request::metric_tag(),
                Tag::new("http_method", Request::method().as_str()),
                Tag::new("status_code", response.status().as_str()),
                Tag::new("base_url", self.base_url),
            ],
            fields: vec![Field::new("duration", duration)],
        };

        if self.metric_tx.send(http_duration).is_err() {
            warn!("failed to send Metric due to dropped channel receiver");
        }

        // Extract Status Code & reqwest::Response Bytes
        let status_code = response.status();
        let payload = response.bytes().await?;// .map_err(SocketError::from)?;

        Ok((status_code, payload))
    }
}