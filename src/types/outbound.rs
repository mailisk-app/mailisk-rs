use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::null_as_default;

/// Sender or reply-to address for outbound email.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailAddress {
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OutboundEmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailAttachmentDisposition {
    Attachment,
    Inline,
}

/// A file attached to an outbound email.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailAttachment {
    pub filename: String,
    pub content_type: String,
    /// File contents encoded as standard base64.
    pub content_base64: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<OutboundEmailAttachmentDisposition>,
}

/// Body for [`MailiskClient::send_email`](crate::MailiskClient::send_email).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SendEmailParams {
    pub subject: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<OutboundEmailAddress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<OutboundEmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<OutboundEmailAttachment>,
}

/// Body for [`MailiskClient::reply_to_email`](crate::MailiskClient::reply_to_email).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplyToEmailParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<OutboundEmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<OutboundEmailAttachment>,
}

/// Body for [`MailiskClient::forward_email`](crate::MailiskClient::forward_email).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ForwardEmailParams {
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<OutboundEmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailType {
    New,
    Reply,
    Forward,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailStatus {
    Queued,
    Sending,
    Sent,
    Failed,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailRecipientType {
    To,
    Cc,
    Bcc,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailRecipientDeliveryStatus {
    Pending,
    Accepted,
    Queued,
    Sent,
    Delayed,
    Deferred,
    Delivered,
    Bounced,
    Expired,
    Complained,
    Unsubscribed,
    Suppressed,
    Rejected,
    Failed,
    Cancelled,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutboundEmailEventType {
    Accepted,
    Queued,
    Sent,
    Failed,
    Rejected,
    Delayed,
    Deferred,
    Delivered,
    Bounced,
    Expired,
    Complained,
    Unsubscribed,
    Suppressed,
    Cancelled,
    Opened,
    Clicked,
    /// A value added to the API after this client version was released.
    #[serde(other)]
    Unknown,
}

/// An outbound email as returned by the send, reply, and forward endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmail {
    pub id: String,
    pub organisation_id: String,
    #[serde(rename = "type")]
    pub kind: OutboundEmailType,
    pub status: OutboundEmailStatus,
    pub from: OutboundEmailAddress,
    #[serde(default)]
    pub reply_to: Option<OutboundEmailAddress>,
    pub subject: String,
    pub recipient_count: u32,
    pub attachment_count: u32,
    pub message_id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub provider_message_id: Option<String>,
    #[serde(default)]
    pub failure_reason: Option<String>,
    pub queued_at: String,
    #[serde(default)]
    pub sending_at: Option<String>,
    #[serde(default)]
    pub sent_at: Option<String>,
    #[serde(default)]
    pub failed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Number of recipients in each delivery state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutboundEmailDeliverySummary {
    pub pending: u32,
    pub accepted: u32,
    pub queued: u32,
    pub sent: u32,
    pub delayed: u32,
    pub deferred: u32,
    pub delivered: u32,
    pub bounced: u32,
    pub expired: u32,
    pub complained: u32,
    pub unsubscribed: u32,
    pub suppressed: u32,
    pub rejected: u32,
    pub failed: u32,
    pub cancelled: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailRecipient {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: OutboundEmailRecipientType,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    pub delivery_status: OutboundEmailRecipientDeliveryStatus,
    #[serde(default)]
    pub last_event_type: Option<String>,
    #[serde(default)]
    pub last_event_at: Option<String>,
    #[serde(default)]
    pub accepted_at: Option<String>,
    #[serde(default)]
    pub queued_at: Option<String>,
    #[serde(default)]
    pub sent_at: Option<String>,
    #[serde(default)]
    pub delayed_at: Option<String>,
    #[serde(default)]
    pub deferred_at: Option<String>,
    #[serde(default)]
    pub delivered_at: Option<String>,
    #[serde(default)]
    pub bounced_at: Option<String>,
    #[serde(default)]
    pub expired_at: Option<String>,
    #[serde(default)]
    pub complained_at: Option<String>,
    #[serde(default)]
    pub unsubscribed_at: Option<String>,
    #[serde(default)]
    pub suppressed_at: Option<String>,
    #[serde(default)]
    pub rejected_at: Option<String>,
    #[serde(default)]
    pub failed_at: Option<String>,
    #[serde(default)]
    pub cancelled_at: Option<String>,
    #[serde(default)]
    pub last_smtp_code: Option<String>,
    #[serde(default)]
    pub last_enhanced_status_code: Option<String>,
    #[serde(default)]
    pub last_smtp_response: Option<String>,
    pub attempt_count: u32,
    #[serde(default)]
    pub bounce_type: Option<String>,
    #[serde(default)]
    pub bounce_subtype: Option<String>,
    #[serde(default)]
    pub bounce_classification: Option<String>,
    #[serde(default)]
    pub complaint_feedback_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailEventRecipient {
    pub id: String,
    #[serde(default)]
    pub outbound_email_recipient_id: Option<String>,
    pub email: String,
    #[serde(default)]
    pub smtp_code: Option<String>,
    #[serde(default)]
    pub enhanced_status_code: Option<String>,
    #[serde(default)]
    pub smtp_response: Option<String>,
    #[serde(default)]
    pub diagnostic_code: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub attempt_count: Option<u32>,
    #[serde(default)]
    pub bounce_classification: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailEvent {
    pub id: String,
    pub provider: String,
    #[serde(default)]
    pub provider_message_id: Option<String>,
    #[serde(default)]
    pub provider_event_id: Option<String>,
    pub event_type: OutboundEmailEventType,
    #[serde(default)]
    pub event_subtype: Option<String>,
    pub occurred_at: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub recipients: Vec<OutboundEmailEventRecipient>,
    pub created_at: String,
}

/// An outbound email with per-recipient delivery state and provider events.
///
/// Dereferences to [`OutboundEmail`] for the shared fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundEmailDetail {
    #[serde(flatten)]
    pub email: OutboundEmail,
    pub delivery_summary: OutboundEmailDeliverySummary,
    #[serde(default, deserialize_with = "null_as_default")]
    pub recipients: Vec<OutboundEmailRecipient>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub events: Vec<OutboundEmailEvent>,
}

impl Deref for OutboundEmailDetail {
    type Target = OutboundEmail;

    fn deref(&self) -> &OutboundEmail {
        &self.email
    }
}
