use crate::{
    error::SocketError,
    metric::{Field, Metric, Tag},
    protocol::http::{
        rest::RestRequest,
        BuildStrategy, HttpParser,
    },
};
use bytes::Bytes;
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::warn;

/// Configurable REST client capable of executing signed [`RestRequest`]s. Use this when
/// integrating APIs that require Http in order to interact with resources. Each API will require
/// a specific combination of [`Signer`], [`Mac`], signature [`Encoder`], and [`Parser`].
#[derive(Debug)]
pub struct RestClient<'a, Strategy, Parser> {
    /// HTTP [`reqwest::Client`] for executing signed [`reqwest::Request`]s.
    pub http_client: reqwest::Client,

    /// Base Url of the API being interacted with.
    pub base_url: &'a str,

    /// [`Metric`] transmitter for sending observed execution measurements to an external receiver.
    pub metric_tx: mpsc::UnboundedSender<Metric>,

    /// [`RestRequest`] build strategy for the API being interacted with that implements
    /// [`BuildStrategy`].
    ///
    /// An authenticated [`RestClient`] will utilise API specific [`signer`] logic, a hashable
    /// [`Mac`], and a signature [`Encoder`]. Where as a non authorised [`RestRequest`] may just
    /// add mandatory [`reqwest::Header`]s.
    pub strategy: Strategy,

    /// [`HttpParser`] that deserialises [`RestRequest::Response`]s, and upon failure parses
    /// API errors returned from the server.
    pub parser: Parser,
}

impl<'a, Strategy, Parser> RestClient<'a, Strategy, Parser>
where
    Strategy: BuildStrategy,
    Parser: HttpParser,
{
    /// Execute the provided [`RestRequest`].
    pub async fn execute<Request>(
        &self,
        request: Request,
    ) -> Result<Request::Response, Parser::OutputError>
    where
        Request: RestRequest,
    {
        // Use provided Request to construct a signed reqwest::Request
        let request = self.build(request)?;

        // Measure request execution
        let (status, payload) = self.measured_execution::<Request>(request).await?;

        // Attempt to parse API Success or Error response
        self.parser.parse::<Request::Response>(status, &payload)
    }

    /// Use the provided [`RestRequest`] to construct a signed Http [`reqwest::Request`].
    pub fn build<Request>(&self, request: Request) -> Result<reqwest::Request, SocketError>
    where
        Request: RestRequest,
    {
        // Construct url
        let url = self.base_url.to_string() + Request::path();

        // Construct RequestBuilder with method & url
        let mut builder = self
            .http_client
            .request(Request::method(), url)
            .timeout(Request::timeout());

        // Add optional query parameters
        if let Some(query_params) = request.query_params() {
            builder = builder.query(query_params);
        }

        // Add optional Body
        if let Some(body) = request.body() {
            builder = builder.json(body);
        }

        // Use RequestBuilder (public or private strategy) to build reqwest::Request
        self.strategy
            .build(request, builder)
    }

    /// Execute the built [`reqwest::Request`] using the [`reqwest::Client`].
    ///
    /// Measures the Http request round trip duration and sends the associated [`Metric`]
    /// via the [`Metric`] transmitter.
    pub async fn measured_execution<Request>(
        &self,
        request: reqwest::Request,
    ) -> Result<(reqwest::StatusCode, Bytes), SocketError>
    where
        Request: RestRequest,
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
        let payload = response.bytes().await?;

        Ok((status_code, payload))
    }
}

impl<'a, Strategy, Parser> RestClient<'a, Strategy, Parser> {
    /// Construct a new [`Self`] using the provided configuration.
    pub fn new(
        base_url: &'a str,
        metric_tx: mpsc::UnboundedSender<Metric>,
        strategy: Strategy,
        parser: Parser,
    ) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url,
            metric_tx,
            strategy,
            parser,
        }
    }
}
