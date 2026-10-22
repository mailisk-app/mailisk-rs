use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::null_as_default;

/// A sender or recipient of a received email.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Attachment metadata embedded in a received email.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub id: String,
    pub filename: String,
    #[serde(default, alias = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    pub size: u64,
}

/// An email received by a namespace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub cc: Vec<EmailAddress>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub bcc: Vec<EmailAddress>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    pub received_date: String,
    /// Unix timestamp in seconds.
    pub received_timestamp: i64,
    /// Unix timestamp in seconds.
    pub expires_timestamp: i64,
    #[serde(default)]
    pub spam_score: Option<f64>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub headers: HashMap<String, String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub attachments: Vec<EmailAttachment>,
}

/// Filters for [`MailiskClient::search_inbox`](crate::MailiskClient::search_inbox).
///
/// The same shape is echoed back as [`SearchInboxResponse::options`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchInboxParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Lower bound as a Unix timestamp in seconds. Defaults to 15 minutes ago.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<i64>,
    /// Upper bound as a Unix timestamp in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_addr_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_addr_includes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_includes: Option<String>,
    /// Wait until at least one email matches. Defaults to `true`; pass
    /// `Some(false)` to return immediately.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchInboxResponse {
    pub total_count: u64,
    pub options: SearchInboxParams,
    pub data: Vec<Email>,
}

/// Attachment metadata including a short-lived download URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachmentDetail {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    #[serde(default)]
    pub expires_at: Option<String>,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetAttachmentResponse {
    pub data: AttachmentDetail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Namespace {
    pub id: String,
    pub namespace: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListNamespacesResponse {
    pub total_count: u64,
    pub data: Vec<Namespace>,
}
