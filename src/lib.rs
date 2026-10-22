//! Typed async Rust client for the [Mailisk](https://mailisk.com) API.
//!
//! Mailisk is an email, SMS, outbound email, and TOTP testing platform. This
//! crate mirrors the Mailisk Node and Python clients: the same endpoints, the
//! same search defaults, and request/response types that match the REST API's
//! JSON fields.
//!
//! ```no_run
//! # async fn run() -> mailisk::Result<()> {
//! use mailisk::{MailiskClient, SearchInboxParams};
//!
//! let mailisk = MailiskClient::new("YOUR_API_KEY")?;
//!
//! // Waits until an email arrives for this address (up to 5 minutes).
//! let result = mailisk
//!     .search_inbox(
//!         "mynamespace",
//!         &SearchInboxParams {
//!             to_addr_prefix: Some("john@mynamespace.mailisk.net".into()),
//!             ..Default::default()
//!         },
//!     )
//!     .await?;
//!
//! for email in result.data {
//!     println!("{:?}", email.subject);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod error;
pub mod types;

pub use client::{MailiskClient, MailiskClientBuilder, RequestOptions};
pub use error::{Error, Result};
pub use types::*;

/// Version of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
