use crate::{error::{SocketError, ExchangeError}};
use std::time::Duration;
use async_trait::async_trait;
use reqwest::RequestBuilder;
use serde::{
    de::DeserializeOwned, Serialize
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub struct Metric<T> {
    name: &'static str,
    tag: &'static str,
    value: T,
}

pub trait HttpRequest<QueryParams = (), Body = ()>
where
    QueryParams: Serialize,
    Body: Serialize,
{
    type Response: DeserializeOwned;

    fn metric_tag() -> &'static str;
    fn method() -> reqwest::Method;

    fn url<S: Into<String>>(&self, base_url: S) -> Result<reqwest::Url, SocketError>
    where
        S: Into<String>
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

    fn path() -> &'static str;
    fn query_params(&self) -> Option<&QueryParams> { None }
    fn body(&self) -> Option<&Body> { None }
}

#[async_trait]
pub trait HttpClient {
    fn client(&self) -> reqwest::Client;
    fn metric_tx(&self) -> mpsc::UnboundedSender<Metric<Duration>>;
    fn base_url(&self) -> &str;

    async fn execute<Request>(&self, request: Request) -> Result<Request::Response, SocketError>
    where
        Request: HttpRequest + Send
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

    fn builder<Request>(&self, request: &Request) -> Result<RequestBuilder, SocketError>
    where
        Request: HttpRequest,
    {
        // Generate Url
        let url = request.url(self.base_url())?;

        // Construct RequestBuilder with method & url
        let mut builder = self
            .client()
            .request(Request::method(), url);

        // Add optional request Body
        if let Some(body) = request.body() {
            builder = builder.json(body);
        }

        Ok(builder)
    }

    fn sign<Request>(&self, _request: &Request, builder: RequestBuilder) -> Result<reqwest::Request, SocketError>
    where
        Request: HttpRequest
    {
        // Default implementation does not sign the request
        // eg/ for public data http request
        builder
            .build()
            .map_err(SocketError::from)
    }

    async fn measured_execution<Request>(&self, request: reqwest::Request) -> Result<reqwest::Response, SocketError>
    where
        Request: HttpRequest
    {
        let start = std::time::Instant::now();
        let response = self.client().execute(request).await?;
        let took = start.elapsed();

        self.metric_tx()
            .send(Metric {
                name: "http_request_duration",
                tag: Request::metric_tag(),
                value: took
            })
            .unwrap();

        println!("Request Time: {took:?}");

        Ok(response)
    }

    async fn parse<Response>(&self, response: reqwest::Response) -> Result<Response, SocketError>
    where
        Response: DeserializeOwned
    {
        // Extract Status Code & reqwest::Response Bytes
        let status_code = response.status();
        let data = response.bytes().await?;

        // Attempt to deserialize reqwest::Response Bytes into Ok(Response)
        let parse_ok_error = match serde_json::from_slice::<Response>(&data) {
            Ok(response) => return Ok(response),
            Err(serde_error) => serde_error,
        };

        // Attempt to deserialise API ExchangeError if Ok(Response) deserialisation failed
        let parse_error_error = match self.parse_error(status_code, &data) {
            Ok(api_error) => return Err(SocketError::Exchange(api_error)),
            Err(serde_error) => serde_error,
        };

        // Log errors if failed to deserialise reqwest::Response into Response or API DriverError
        error!(
            ?status_code,
            ?parse_ok_error,
            ?parse_error_error,
            response_body = %String::from_utf8_lossy(&data),
            "error deserializing HTTP response"
        );

        Err(SocketError::DeserialiseBinary {
            error: parse_ok_error,
            payload: data.to_vec()
        })
    }

    fn parse_error(&self, status_code: StatusCode, data: &[u8]) -> Result<ExchangeError, SocketError>;
}