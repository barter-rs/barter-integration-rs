use crate::{
    metric::{Field, Metric, Tag},
    SocketError
};
use super::{
    HttpParser,
    signer::Signer
};
use std::time::Duration;
use bytes::Bytes;
use chrono::Utc;
use reqwest::{RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::warn;
use crate::protocol::http_old::HttpRequest;

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

    /// [`Metric`] [`Tag`] that identifies this request.
    fn metric_tag() -> Tag;

    /// Http [`reqwest::Method`] of this request.
    fn method() -> reqwest::Method;

    /// Additional [`Url`] path to the resource.
    fn path() -> &'static str;

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

/// Todo:
#[derive(Debug)]
pub struct RestClient<'a, Sig, Parser> {
    base_url: &'a str,
    http_client: reqwest::Client,
    metric_tx: mpsc::UnboundedSender<Metric>,
    signer: Sig,
    parser: Parser,
}

impl<'a, Sign, Parser> RestClient<'a, Sign, Parser>
where
    Sign: Signer,
    Parser: HttpParser,
{
    // Could pass different builders here for each request?
    async fn execute<Request>(&self, request: Request) -> Result<Request::Response, Parser::Error>
    where
        Request: RestRequest,
    {
        // Use provided Request to construct a reqwest::RequestBuilder
        let builder = self.builder(&request)?;

        // Sign:
        // 1. Generate signature
        // 2. Add to request
        // 3. build request

        // Sign reqwest::RequestBuilder with exchange specific recipe
        let request = self.signer.sign(&request, builder)?;

        // Measure request execution
        let (status, payload) = self
            .measured_execution::<Request>(request)
            .await?;

        // Attempt to parse API Success or Error response
        self.parser
            .parse::<Request::Response>(status, &payload)
    }

    // Todo: This is duplicated. In RestRequest or here?
    fn url<Request>(&self, request: &Request) -> Result<reqwest::Url, SocketError>
    where
        Request: RestRequest
    {
        // Generate Url String
        let mut url = self.base_url.to_owned() + Request::path();

        // Add optional query parameters
        if let Some(parameters) = request.query_params() {
            let query_string = serde_qs::to_string(parameters)?;
            url.push('?');
            url.push_str(&query_string);
        }

        reqwest::Url::parse(&url).map_err(SocketError::from)
    }

    /// Use the provided [`RestRequest`] to construct a Http [`reqwest::RequestBuilder`].
    fn builder<Request>(&self, request: &Request) -> Result<RequestBuilder, SocketError>
    where
        Request: RestRequest
    {
        // Generate Url
        let url = request.url(base_url)?;

        // Construct RequestBuilder with method & url
        let mut builder = self
            .http_client
            .request(Request::method(), url)
            .timeout(Request::timeout());

        // Add optional request Body
        if let Some(body) = request.body() {
            builder = builder.json(body);
        }

        Ok(builder)
    }

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