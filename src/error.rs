use reqwest::StatusCode;

/// Errors returned by [`MailiskClient`](crate::MailiskClient).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The client was configured with invalid values, such as an empty API key
    /// or an unparsable base URL.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// The request could not be sent or the response could not be read
    /// (connection failures, timeouts, too many redirects).
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// The API responded with a non-success status code.
    #[error("Mailisk API returned {status}: {body}")]
    Api {
        status: StatusCode,
        /// Raw response body, usually a JSON error object.
        body: String,
    },

    /// The response body could not be decoded into the expected type.
    #[error("failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),
}

impl Error {
    /// The HTTP status code, when the error came from an API or download response.
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Error::Api { status, .. } => Some(*status),
            Error::Request(error) => error.status(),
            _ => None,
        }
    }
}

/// Convenience alias for results returned by this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
