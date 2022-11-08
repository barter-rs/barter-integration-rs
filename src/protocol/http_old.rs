use std::fmt::Debug;
use crate::{
    error::SocketError,
    metric::{Field, Metric, Tag},
};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::http::StatusCode;
use tracing::{error, warn};

/// Default Http [`reqwest::Request`] timeout Duration.
const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Http client that executes [`HttpRequest`]s. Implement this when integrating APIs that require
/// Http to interact with server resources.
#[async_trait]
pub trait HttpClient {
    type Error: From<SocketError> + Debug;

    /// Reference to a reusable [`reqwest::Client`].
    fn client(&self) -> &reqwest::Client;

    /// Reference to the [`Metric`] channel transmitter, used for sending measured Http request
    /// execution metadata.
    fn metric_tx(&self) -> &mpsc::UnboundedSender<Metric>;

    /// Base Url of the API being interacted with.
    fn base_url(&self) -> &str;

    /// Execute the provided [`HttpRequest`].
    async fn execute<Request>(&self, request: Request) -> Result<Request::Response, Self::Error>
    where
        Request: HttpRequest + Send,
    {
        // Use provided Request to construct a reqwest::RequestBuilder
        let builder = self.builder(&request)?;

        // Sign reqwest::RequestBuilder with exchange specific recipe
        let request = self.sign(&request, builder)?;

        // Measure request execution
        let response = self.measured_execution::<Request>(request).await?;

        // Attempt to parse API Success or Error response
        self.parse::<Request::Response>(response)
            .await
    }

    /// Use the provided [`HttpRequest`] to construct a Http [`reqwest::RequestBuilder`].
    fn builder<Request>(&self, request: &Request) -> Result<RequestBuilder, SocketError>
    where
        Request: HttpRequest,
    {
        // Generate Url
        let url = request.url(self.base_url())?;

        // Construct RequestBuilder with method & url
        let mut builder = self
            .client()
            .request(Request::method(), url)
            .timeout(Request::timeout());

        // Add optional request Body
        if let Some(body) = request.body() {
            builder = builder.json(body);
        }

        Ok(builder)
    }

    /// Sign the outgoing Http request and add any required API specific headers.
    ///
    /// # Examples
    ///
    /// ## Public Http Request
    ///  - No signing required.
    ///  - No additional headers required.
    ///
    /// ```rust,ignore
    /// fn sign<Request>(&self, _: &Request, builder: RequestBuilder) -> Result<Reqwest::Request, SocketError>
    /// where
    ///     Request: HttpRequest,
    /// {
    ///     builder
    ///         .build()
    ///         .map_err(SocketError::from)
    /// }
    /// ```
    ///
    /// ## Private Http Request: Ftx GET Request
    /// - Hmac signing.
    /// - Added Ftx required headers.
    ///
    /// ```rust,ignore
    /// fn sign<Request>(&self, _: &Request, builder: RequestBuilder) -> Result<Reqwest::Request, SocketError>
    /// where
    ///     Request: HttpRequest,
    /// {
    ///     // Current millisecond timestamp
    ///     let time = Utc::now().timestamp_millis().to_string();
    ///
    ///     // Generate signature
    ///     let mut hmac = self.hmac.clone();
    ///     hmac.update(format!("{time}{}{}", Request::method(), Request::path()).as_bytes());
    ///     let signature = format!("{:x}", hmac.finalize().into_bytes());
    ///
    ///     // Add Ftx required Headers & build reqwest::Request
    ///     builder
    ///         .header(HEADER_FTX_KEY, &self.api_key)
    ///         .header(HEADER_FTX_SIGN, &signature)
    ///         .header(HEADER_FTX_TS, &time)
    ///         .build()
    ///         .map_err(SocketError::from)
    /// }
    /// ```
    fn sign<Request>(
        &self,
        request: &Request,
        builder: RequestBuilder,
    ) -> Result<reqwest::Request, SocketError>
    where
        Request: HttpRequest;

    /// Execute the built [`reqwest::Request`] using the [`reqwest::Client`].
    ///
    /// Default implementation measures the Http request round trip duration and sends the
    /// associated [`Metric`] on the [`Metric`] transmitter.
    async fn measured_execution<Request>(
        &self,
        request: reqwest::Request,
    ) -> Result<reqwest::Response, SocketError>
    where
        Request: HttpRequest,
    {
        // Measure the HTTP request round trip duration
        let start = std::time::Instant::now();
        let response = self.client().execute(request).await?;
        let duration = start.elapsed().as_millis() as u64;

        // Construct HTTP request duration Metric & send
        let http_duration = Metric {
            name: "http_request_duration",
            time: Utc::now().timestamp_millis() as u64,
            tags: vec![
                Request::metric_tag(),
                Tag::new("http_method", Request::method().as_str()),
                Tag::new("status_code", response.status().as_str()),
                Tag::new("base_url", self.base_url()),
            ],
            fields: vec![Field::new("duration", duration)],
        };

        if self.metric_tx().send(http_duration).is_err() {
            warn!(
                why = "Metric channel receiver dropped",
                "failed to send Metric"
            );
        }

        Ok(response)
    }

    /// Attempt to parse the [`reqwest::Response`] into the associated [`HttpRequest::Response`].
    async fn parse<Response>(&self, response: reqwest::Response) -> Result<Response, Self::Error>
    where
        Response: DeserializeOwned,
    {
        // Extract Status Code & reqwest::Response Bytes
        let status_code = response.status();
        let data = response.bytes().await.map_err(SocketError::from)?;

        // Attempt to deserialize reqwest::Response Bytes into Ok(Response)
        let parse_ok_error = match serde_json::from_slice::<Response>(&data) {
            Ok(response) => return Ok(response),
            Err(serde_error) => serde_error,
        };

        // Attempt to deserialise API ExchangeError if Ok(Response) deserialisation failed
        let parse_error_error = match self.parse_error(status_code, &data) {
            Ok(api_error) => return Err(api_error),
            Err(serde_error) => serde_error,
        };

        // Log errors if failed to deserialise reqwest::Response into Response or API Self::Error
        error!(
            ?status_code,
            ?parse_ok_error,
            ?parse_error_error,
            response_body = %String::from_utf8_lossy(&data),
            "error deserializing HTTP response"
        );

        Err(Self::Error::from(SocketError::DeserialiseBinary {
            error: parse_ok_error,
            payload: data.to_vec(),
        }))
    }

    /// If [`parse`](Self::parse) fails, this function attempts to parse the normalised
    /// [`SocketError::Exchange`](ExchangeError) associated with the response.
    fn parse_error(&self, status: StatusCode, data: &[u8]) -> Result<Self::Error, SocketError>;
}

/// Http request that can be executed by a [`HttpClient`].
pub trait HttpRequest {
    /// Serialisable query parameters type - use unit struct () if not required for this request.
    type QueryParams: Serialize;

    /// Serialisable Body type - use unit struct () if not required for this request.
    type Body: Serialize;

    /// Expected response type this request will be answered with if successful.
    type Response: DeserializeOwned;

    /// [`Metric`] [`Tag`] that identifies this request.
    fn metric_tag() -> Tag;

    /// Additional [`Url`] path to the resource.
    fn path() -> &'static str;

    /// Http [`reqwest::Method`] of this request.
    fn method() -> reqwest::Method;

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
