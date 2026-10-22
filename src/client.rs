use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderValue, USER_AGENT};
use reqwest::{Method, RequestBuilder, Url, redirect};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::types::*;

const API_KEY_HEADER: &str = "X-Api-Key";
const CLIENT_USER_AGENT: &str = concat!("mailisk-rs/", env!("CARGO_PKG_VERSION"));

/// Per-request options for the waiting search endpoints and attachment downloads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RequestOptions {
    /// Total time allowed for the request, including any wait redirects.
    ///
    /// When unset, waiting searches use [`MailiskClient::DEFAULT_WAIT_TIMEOUT`]
    /// and every other request uses the HTTP client's own timeout.
    pub timeout: Option<Duration>,
}

impl RequestOptions {
    pub fn timeout(timeout: Duration) -> Self {
        Self { timeout: Some(timeout) }
    }
}

/// Async client for the Mailisk API.
///
/// Cloning is cheap and shares the underlying connection pool.
///
/// ```no_run
/// # async fn run() -> mailisk::Result<()> {
/// use mailisk::{MailiskClient, SearchInboxParams};
///
/// let mailisk = MailiskClient::new("YOUR_API_KEY")?;
/// let result = mailisk
///     .search_inbox("mynamespace", &SearchInboxParams::default())
///     .await?;
/// println!("{} emails", result.data.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MailiskClient {
    http: reqwest::Client,
    api_key: HeaderValue,
    base_url: Url,
}

/// Builder for [`MailiskClient`], created with [`MailiskClient::builder`].
#[derive(Debug)]
pub struct MailiskClientBuilder {
    api_key: String,
    base_url: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl MailiskClientBuilder {
    /// Overrides the API URL, for example `https://api.mailisk.dev`.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Uses a preconfigured HTTP client, for example to set a proxy.
    ///
    /// Waiting searches are implemented by the API as repeated redirects to the
    /// same URL, so the client's redirect policy must allow enough redirects to
    /// cover the wait. The default client allows
    /// [`MailiskClient::DEFAULT_MAX_REDIRECTS`].
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn build(self) -> Result<MailiskClient> {
        if self.api_key.is_empty() {
            return Err(Error::Config("api_key is required".into()));
        }

        let mut api_key = HeaderValue::from_str(&self.api_key)
            .map_err(|_| Error::Config("api_key contains invalid header characters".into()))?;
        api_key.set_sensitive(true);

        let base_url = self.base_url.as_deref().unwrap_or(MailiskClient::DEFAULT_BASE_URL);
        let base_url =
            Url::parse(base_url).map_err(|error| Error::Config(format!("invalid base_url {base_url:?}: {error}")))?;
        if base_url.cannot_be_a_base() {
            return Err(Error::Config(format!("invalid base_url {base_url:?}")));
        }

        let http = match self.http_client {
            Some(http) => http,
            None => reqwest::Client::builder()
                .redirect(redirect::Policy::limited(MailiskClient::DEFAULT_MAX_REDIRECTS))
                .build()?,
        };

        Ok(MailiskClient {
            http,
            api_key,
            base_url,
        })
    }
}

impl MailiskClient {
    pub const DEFAULT_BASE_URL: &'static str = "https://api.mailisk.com/";
    /// How far back searches look when no lower bound is given.
    pub const DEFAULT_LOOKBACK: Duration = Duration::from_secs(15 * 60);
    /// Timeout for waiting searches when no timeout is given.
    pub const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(5 * 60);
    pub const DEFAULT_MAX_REDIRECTS: usize = 99_999;

    /// Creates a client for the production API.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    pub fn builder(api_key: impl Into<String>) -> MailiskClientBuilder {
        MailiskClientBuilder {
            api_key: api_key.into(),
            base_url: None,
            http_client: None,
        }
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    // Email

    /// Searches a namespace's inbox, newest first.
    ///
    /// Unless overridden, only emails from the last 15 minutes are searched and
    /// the request waits (up to 5 minutes) until at least one email matches.
    pub async fn search_inbox(&self, namespace: &str, params: &SearchInboxParams) -> Result<SearchInboxResponse> {
        self.search_inbox_with_options(namespace, params, &RequestOptions::default())
            .await
    }

    pub async fn search_inbox_with_options(
        &self,
        namespace: &str,
        params: &SearchInboxParams,
        options: &RequestOptions,
    ) -> Result<SearchInboxResponse> {
        let mut params = params.clone();
        if params.from_timestamp.is_none() {
            let lookback = SystemTime::now() - Self::DEFAULT_LOOKBACK;
            params.from_timestamp = Some(unix_seconds(lookback));
        }
        let wait = *params.wait.get_or_insert(true);

        let request = self
            .request(Method::GET, &["api", "emails", namespace, "inbox"])
            .query(&params);
        self.send_json(with_wait_timeout(request, wait, options)).await
    }

    pub async fn list_namespaces(&self) -> Result<ListNamespacesResponse> {
        self.send_json(self.request(Method::GET, &["api", "namespaces"])).await
    }

    pub async fn get_attachment(&self, attachment_id: &str) -> Result<GetAttachmentResponse> {
        self.send_json(self.request(Method::GET, &["api", "attachments", attachment_id]))
            .await
    }

    /// Downloads an attachment's contents through its presigned URL.
    pub async fn download_attachment(&self, attachment_id: &str) -> Result<Vec<u8>> {
        self.download_attachment_with_options(attachment_id, &RequestOptions::default())
            .await
    }

    pub async fn download_attachment_with_options(
        &self,
        attachment_id: &str,
        options: &RequestOptions,
    ) -> Result<Vec<u8>> {
        let attachment = self.get_attachment(attachment_id).await?;

        // The presigned URL is not a Mailisk API endpoint, so the API key is not sent.
        let mut request = self
            .http
            .get(&attachment.data.download_url)
            .header(USER_AGENT, CLIENT_USER_AGENT);
        if let Some(timeout) = options.timeout {
            request = request.timeout(timeout);
        }

        let response = check_status(request.send().await?).await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn send_email(&self, namespace: &str, params: &SendEmailParams) -> Result<OutboundEmail> {
        let request = self
            .request(Method::POST, &["api", "emails", "send"])
            .query(&[("namespace", namespace)])
            .json(params);
        self.send_json(request).await
    }

    pub async fn get_outbound_email(&self, outbound_email_id: &str) -> Result<OutboundEmailDetail> {
        self.send_json(self.request(Method::GET, &["api", "emails", "outbound", outbound_email_id]))
            .await
    }

    pub async fn reply_to_email(&self, email_id: &str, params: &ReplyToEmailParams) -> Result<OutboundEmail> {
        let request = self
            .request(Method::POST, &["api", "emails", email_id, "reply"])
            .json(params);
        self.send_json(request).await
    }

    pub async fn forward_email(&self, email_id: &str, params: &ForwardEmailParams) -> Result<OutboundEmail> {
        let request = self
            .request(Method::POST, &["api", "emails", email_id, "forward"])
            .json(params);
        self.send_json(request).await
    }

    // SMS

    /// Searches SMS messages for a phone number, newest first.
    ///
    /// Unless overridden, only messages from the last 15 minutes are searched
    /// and the request waits (up to 5 minutes) until at least one message matches.
    pub async fn search_sms_messages(
        &self,
        phone_number: &str,
        params: &SearchSmsMessagesParams,
    ) -> Result<SearchSmsMessagesResponse> {
        self.search_sms_messages_with_options(phone_number, params, &RequestOptions::default())
            .await
    }

    pub async fn search_sms_messages_with_options(
        &self,
        phone_number: &str,
        params: &SearchSmsMessagesParams,
        options: &RequestOptions,
    ) -> Result<SearchSmsMessagesResponse> {
        let mut params = params.clone();
        if params.from_date.is_none() {
            let lookback = SystemTime::now() - Self::DEFAULT_LOOKBACK;
            params.from_date = Some(iso8601_utc(lookback));
        }
        let wait = *params.wait.get_or_insert(true);

        let request = self
            .request(Method::GET, &["api", "sms", phone_number, "messages"])
            .query(&params);
        self.send_json(with_wait_timeout(request, wait, options)).await
    }

    pub async fn list_sms_numbers(&self) -> Result<ListSmsNumbersResponse> {
        self.send_json(self.request(Method::GET, &["api", "sms", "numbers"]))
            .await
    }

    /// Creates an inbound SMS on one of your numbers without a carrier.
    pub async fn send_virtual_sms(&self, params: &SendVirtualSmsParams) -> Result<()> {
        let request = self.request(Method::POST, &["api", "sms", "virtual"]).json(params);
        self.send_empty(request).await
    }

    // TOTP

    pub async fn list_totp_devices(&self, params: &ListTotpDevicesParams) -> Result<ListTotpDevicesResponse> {
        let mut params = params.clone();
        for value in [&mut params.username, &mut params.issuer].into_iter().flatten() {
            *value = value.trim().to_owned();
        }

        let request = self.request(Method::GET, &["api", "devices"]).query(&params);
        self.send_json(request).await
    }

    pub async fn create_totp_device(&self, params: &CreateTotpDeviceParams) -> Result<TotpDevice> {
        self.send_json(self.request(Method::POST, &["api", "devices"]).json(params))
            .await
    }

    pub async fn create_custom_totp_device(&self, params: &CreateCustomTotpDeviceParams) -> Result<TotpDevice> {
        self.send_json(self.request(Method::POST, &["api", "devices", "custom"]).json(params))
            .await
    }

    pub async fn create_totp_device_from_base32_secret_key(
        &self,
        params: &CreateBase32SecretKeyTotpDeviceParams,
    ) -> Result<TotpDevice> {
        let request = self
            .request(Method::POST, &["api", "devices", "base32-secret-key"])
            .json(params);
        self.send_json(request).await
    }

    pub async fn create_totp_device_from_otp_auth_url(
        &self,
        params: &CreateOtpAuthUrlTotpDeviceParams,
    ) -> Result<TotpDevice> {
        let request = self
            .request(Method::POST, &["api", "devices", "otpauth-url"])
            .json(params);
        self.send_json(request).await
    }

    pub async fn get_totp_otp_by_shared_secret(
        &self,
        shared_secret: &str,
        params: &GetTotpOtpParams,
    ) -> Result<TotpOtpResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            shared_secret: &'a str,
            #[serde(flatten)]
            params: &'a GetTotpOtpParams,
        }

        let request = self
            .request(Method::POST, &["api", "devices", "otp"])
            .json(&Body { shared_secret, params });
        self.send_json(request).await
    }

    pub async fn get_totp_otp_by_device_id(
        &self,
        device_id: &str,
        params: &GetTotpOtpParams,
    ) -> Result<TotpOtpResponse> {
        let request = self
            .request(Method::GET, &["api", "devices", device_id, "otp"])
            .query(params);
        self.send_json(request).await
    }

    pub async fn delete_totp_device(&self, device_id: &str) -> Result<()> {
        self.send_empty(self.request(Method::DELETE, &["api", "devices", device_id]))
            .await
    }

    // Transport

    fn request(&self, method: Method, segments: &[&str]) -> RequestBuilder {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("base_url is validated when the client is built")
            .pop_if_empty()
            .extend(segments);

        self.http
            .request(method, url)
            .header(API_KEY_HEADER, self.api_key.clone())
            .header(USER_AGENT, CLIENT_USER_AGENT)
    }

    async fn send_json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        let response = check_status(request.send().await?).await?;
        let body = response.bytes().await?;
        Ok(serde_json::from_slice(&body)?)
    }

    async fn send_empty(&self, request: RequestBuilder) -> Result<()> {
        check_status(request.send().await?).await?;
        Ok(())
    }
}

fn with_wait_timeout(request: RequestBuilder, wait: bool, options: &RequestOptions) -> RequestBuilder {
    match options.timeout {
        Some(timeout) => request.timeout(timeout),
        None if wait => request.timeout(MailiskClient::DEFAULT_WAIT_TIMEOUT),
        None => request,
    }
}

async fn check_status(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    Err(Error::Api { status, body })
}

fn unix_seconds(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// Formats a time as `YYYY-MM-DDTHH:MM:SS.sssZ`.
fn iso8601_utc(time: SystemTime) -> String {
    let since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since_epoch.as_secs();
    let (year, month, day) = civil_from_days((seconds / 86_400) as i64);
    let second_of_day = seconds % 86_400;

    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        second_of_day / 3600,
        second_of_day % 3600 / 60,
        second_of_day % 60,
        since_epoch.subsec_millis(),
    )
}

/// Converts days since 1970-01-01 to a proleptic Gregorian (year, month, day).
/// See <https://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_iso8601_with_milliseconds() {
        let cases = [
            (0, 0, "1970-01-01T00:00:00.000Z"),
            (951_782_400, 5, "2000-02-29T00:00:00.005Z"),
            (1_778_522_400, 123, "2026-05-11T18:00:00.123Z"),
            (4_102_444_799, 999, "2099-12-31T23:59:59.999Z"),
        ];

        for (seconds, millis, expected) in cases {
            let time = UNIX_EPOCH + Duration::from_secs(seconds) + Duration::from_millis(millis);
            assert_eq!(iso8601_utc(time), expected);
        }
    }
}
