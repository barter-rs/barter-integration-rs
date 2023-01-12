use crate::metric::Tag;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Configurable [`client::RestClient`] capable of executing signed [`RestRequest`]s and parsing
/// responses.
pub mod client;

/// Default Http [`reqwest::Request`] timeout Duration.
const DEFAULT_HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Http REST request that can be executed by a [`RestClient`](self::client::RestClient).
pub trait RestRequest {
    /// Expected response type if this request was successful.
    type Response: DeserializeOwned;

    /// Serialisable query parameters type - use unit struct () if not required for this request.
    type QueryParams: Serialize;

    /// Serialisable Body type - use unit struct () if not required for this request.
    type Body: Serialize;

    /// Additional [`Url`](url::Url) path to the resource.
    fn path() -> &'static str;

    /// Http [`reqwest::Method`] of this request.
    fn method() -> reqwest::Method;

    /// [`Metric`](crate::metric::Metric) [`Tag`](crate::metric::Tag) that identifies this request.
    fn metric_tag() -> Tag;

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
