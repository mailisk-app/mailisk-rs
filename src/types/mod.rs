//! Request parameters and response bodies for the Mailisk API.
//!
//! Field names match the JSON field names of the REST API. Optional request
//! fields are omitted from the request when they are `None`.

mod email;
mod outbound;
mod sms;
mod totp;

pub use email::*;
pub use outbound::*;
pub use sms::*;
pub use totp::*;

use serde::{Deserialize, Deserializer};

/// Deserializes a missing or `null` value as `T::default()`.
pub(crate) fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
