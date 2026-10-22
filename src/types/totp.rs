use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TotpAlgorithm {
    #[serde(rename = "SHA1")]
    Sha1,
    #[serde(rename = "SHA256")]
    Sha256,
    #[serde(rename = "SHA512")]
    Sha512,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TotpDevice {
    pub id: String,
    pub organisation_id: String,
    pub name: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
    pub digits: u8,
    pub period: u32,
    pub algorithm: TotpAlgorithm,
    /// How the device was created, for example `shared_secret`, `custom`,
    /// `base32_secret_key`, or `otpauth_url`.
    pub source: String,
    #[serde(default)]
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Filters for [`MailiskClient::list_totp_devices`](crate::MailiskClient::list_totp_devices).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ListTotpDevicesParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTotpDevicesResponse {
    pub total_count: u64,
    pub options: ListTotpDevicesParams,
    pub items: Vec<TotpDevice>,
}

/// Body for [`MailiskClient::create_totp_device`](crate::MailiskClient::create_totp_device).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateTotpDeviceParams {
    pub shared_secret: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Device settings shared by the custom, base32, and otpauth URL create endpoints.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TotpDeviceOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// `6` or `8`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digits: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<TotpAlgorithm>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Body for [`MailiskClient::create_custom_totp_device`](crate::MailiskClient::create_custom_totp_device).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateCustomTotpDeviceParams {
    pub secret: String,
    #[serde(flatten)]
    pub options: TotpDeviceOptions,
}

/// Body for [`MailiskClient::create_totp_device_from_base32_secret_key`](crate::MailiskClient::create_totp_device_from_base32_secret_key).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateBase32SecretKeyTotpDeviceParams {
    pub base32_secret_key: String,
    #[serde(flatten)]
    pub options: TotpDeviceOptions,
}

/// Body for [`MailiskClient::create_totp_device_from_otp_auth_url`](crate::MailiskClient::create_totp_device_from_otp_auth_url).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CreateOtpAuthUrlTotpDeviceParams {
    pub otp_auth_url: String,
    #[serde(flatten)]
    pub options: TotpDeviceOptions,
}

/// Options for the TOTP code endpoints.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GetTotpOtpParams {
    /// Wait for the next code when the current one expires sooner than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_seconds_until_expire: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TotpOtpResponse {
    pub code: String,
    pub expires: String,
}
