use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SmsMessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum SmsNumberStatus {
    Requested,
    Active,
    Disabled,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsMessage {
    pub id: String,
    pub sms_phone_number_id: String,
    pub body: String,
    pub from_number: String,
    pub to_number: String,
    #[serde(default)]
    pub provider_message_id: Option<String>,
    pub created_at: String,
    pub direction: SmsMessageDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsNumber {
    pub id: String,
    pub organisation_id: String,
    pub status: SmsNumberStatus,
    pub country: String,
    #[serde(default)]
    pub phone_number: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Filters for [`MailiskClient::search_sms_messages`](crate::MailiskClient::search_sms_messages).
///
/// The same shape is echoed back as [`SearchSmsMessagesResponse::options`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchSmsMessagesParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Case-insensitive body substring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_number: Option<String>,
    /// Exact message direction. Omit to search both directions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<SmsMessageDirection>,
    /// Inclusive ISO 8601 lower bound. Defaults to 15 minutes ago.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    /// Inclusive ISO 8601 upper bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    /// Wait until at least one message matches. Defaults to `true`; pass
    /// `Some(false)` to return immediately.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchSmsMessagesResponse {
    pub total_count: u64,
    pub options: SearchSmsMessagesParams,
    pub data: Vec<SmsMessage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListSmsNumbersResponse {
    pub total_count: u64,
    pub data: Vec<SmsNumber>,
}

/// Body for [`MailiskClient::send_virtual_sms`](crate::MailiskClient::send_virtual_sms).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendVirtualSmsParams {
    pub from_number: String,
    pub to_number: String,
    pub body: String,
}
