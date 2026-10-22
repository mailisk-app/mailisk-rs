# Mailisk Rust Client

Typed async Rust client for the Mailisk REST API.

Mailisk is an email, SMS, outbound email, and TOTP testing platform. This crate
mirrors the Mailisk Node and Python clients: the same endpoints, the same search
defaults, and request/response structs whose fields match the API's JSON fields.

## Installation

```shell
cargo add mailisk
cargo add tokio --features macros,rt-multi-thread
```

The client is async and built on `reqwest`, so it needs a Tokio runtime.
TLS uses `rustls` by default. To use the platform TLS library instead:

```toml
mailisk = { version = "0.1", default-features = false, features = ["native-tls"] }
```

The minimum supported Rust version is 1.88.

## Create A Client

Get your API key from Mailisk and pass it directly when creating a client:

```rust
use mailisk::MailiskClient;

let mailisk = MailiskClient::new("YOUR_API_KEY")?;
```

The library does not read `.env` files or environment variables itself. Load
configuration however your application prefers, then pass it to the client.
Use the builder to target another API URL or to supply your own `reqwest::Client`:

```rust
let mailisk = MailiskClient::builder(api_key)
    .base_url("https://api.mailisk.com")
    .build()?;
```

`MailiskClient` is cheap to clone; clones share one connection pool.

## Quick Start

```rust
use mailisk::{MailiskClient, SearchInboxParams};

#[tokio::main]
async fn main() -> mailisk::Result<()> {
    let mailisk = MailiskClient::new("YOUR_API_KEY")?;

    let result = mailisk
        .search_inbox("mynamespace", &SearchInboxParams::default())
        .await?;
    println!("{:#?}", result.data);

    Ok(())
}
```

Request parameter structs implement `Default`, so you only set the fields you
need and finish with `..Default::default()`.

## Defaults

`search_inbox` and `search_sms_messages` match the Node and Python client defaults:

- wait for at least one matching message unless `wait` is `Some(false)`
- ignore messages older than 15 minutes unless a lower bound is provided
- use a 5 minute timeout while waiting
- allow a high redirect count, because the API implements waiting by
  redirecting back to the same URL

Pass a different timeout with the `_with_options` variants:

```rust
use std::time::Duration;
use mailisk::{RequestOptions, SearchInboxParams};

mailisk
    .search_inbox_with_options(
        "mynamespace",
        &SearchInboxParams {
            to_addr_prefix: Some("john@mynamespace.mailisk.net".into()),
            ..Default::default()
        },
        &RequestOptions::timeout(Duration::from_secs(60)),
    )
    .await?;
```

If you supply your own `reqwest::Client`, configure its redirect policy to
allow enough redirects to cover a full wait, for example
`reqwest::redirect::Policy::limited(MailiskClient::DEFAULT_MAX_REDIRECTS)`.

## Email

Search the inbox:

```rust
let response = mailisk
    .search_inbox(
        "mynamespace",
        &SearchInboxParams {
            to_addr_prefix: Some("john@mynamespace.mailisk.net".into()),
            subject_includes: Some("password".into()),
            ..Default::default()
        },
    )
    .await?;

let emails = response.data;
```

List namespaces:

```rust
let namespaces = mailisk.list_namespaces().await?;
for namespace in namespaces.data {
    println!("{}", namespace.namespace);
}
```

Download an attachment:

```rust
let attachment_id = &emails[0].attachments[0].id;
let content: Vec<u8> = mailisk.download_attachment(attachment_id).await?;
```

## Outbound Email

Send an outbound email:

```rust
use mailisk::{OutboundEmailAddress, SendEmailParams};

let email = mailisk
    .send_email(
        "mynamespace",
        &SendEmailParams {
            from: Some(OutboundEmailAddress::with_name("support@mynamespace.mailisk.net", "Support")),
            to: vec!["verified@example.com".into()],
            subject: "Hello from Mailisk".into(),
            text: Some("Plain text body".into()),
            html: Some("<p>HTML body</p>".into()),
            ..Default::default()
        },
    )
    .await?;

println!("{} {:?}", email.id, email.status);
```

Send with an attachment. Attachment contents are base64 encoded, for example
with the `base64` crate:

```rust
use base64::Engine;
use mailisk::OutboundEmailAttachment;

let content = std::fs::read("report.txt")?;

mailisk
    .send_email(
        "mynamespace",
        &SendEmailParams {
            to: vec!["verified@example.com".into()],
            subject: "Report".into(),
            text: Some("Attached.".into()),
            attachments: vec![OutboundEmailAttachment {
                filename: "report.txt".into(),
                content_type: "application/octet-stream".into(),
                content_base64: base64::engine::general_purpose::STANDARD.encode(content),
                content_id: None,
                disposition: None,
            }],
            ..Default::default()
        },
    )
    .await?;
```

Fetch delivery details:

```rust
let detail = mailisk.get_outbound_email(&email.id).await?;
println!("{:?}", detail.delivery_summary);
```

Reply to an inbound email:

```rust
use mailisk::ReplyToEmailParams;

let reply = mailisk
    .reply_to_email(
        &emails[0].id,
        &ReplyToEmailParams {
            subject: Some("Re: Thanks".into()),
            text: Some("We received your message.".into()),
            ..Default::default()
        },
    )
    .await?;
```

Forward an inbound email:

```rust
use mailisk::ForwardEmailParams;

let forwarded = mailisk
    .forward_email(
        &emails[0].id,
        &ForwardEmailParams {
            to: vec!["verified@example.com".into()],
            subject: Some("Fwd: Support request".into()),
            text: Some("Forwarding this along.".into()),
            ..Default::default()
        },
    )
    .await?;
```

## SMS

Search SMS messages:

```rust
use mailisk::SearchSmsMessagesParams;

let messages = mailisk
    .search_sms_messages(
        "+15551234567",
        &SearchSmsMessagesParams {
            from_number: Some("+1800555".into()),
            body: Some("Your code".into()),
            ..Default::default()
        },
    )
    .await?;
```

List SMS numbers:

```rust
let numbers = mailisk.list_sms_numbers().await?;
```

Send a virtual SMS:

```rust
use mailisk::SendVirtualSmsParams;

mailisk
    .send_virtual_sms(&SendVirtualSmsParams {
        from_number: "+15550000001".into(),
        to_number: "+15550000002".into(),
        body: "Test message".into(),
    })
    .await?;
```

## TOTP

Create a saved TOTP device:

```rust
use mailisk::CreateTotpDeviceParams;

let device = mailisk
    .create_totp_device(&CreateTotpDeviceParams {
        name: Some("GitHub staging".into()),
        shared_secret: "JBSWY3DPEHPK3PXP".into(),
        ..Default::default()
    })
    .await?;
```

Generate a TOTP code from a shared secret:

```rust
use mailisk::GetTotpOtpParams;

let otp = mailisk
    .get_totp_otp_by_shared_secret(
        "JBSWY3DPEHPK3PXP",
        &GetTotpOtpParams {
            min_seconds_until_expire: Some(10),
        },
    )
    .await?;

println!("{}", otp.code);
```

Generate a TOTP code for a saved device:

```rust
let otp = mailisk
    .get_totp_otp_by_device_id(&device.id, &GetTotpOtpParams::default())
    .await?;
```

Delete a saved TOTP device:

```rust
mailisk.delete_totp_device(&device.id).await?;
```

## Errors

Every method returns `mailisk::Result<T>`. A non-success API response becomes
`Error::Api` with the status code and the raw response body:

```rust
match mailisk.list_namespaces().await {
    Ok(namespaces) => println!("{}", namespaces.total_count),
    Err(mailisk::Error::Api { status, body }) => eprintln!("API error {status}: {body}"),
    Err(error) => eprintln!("{error}"),
}
```

## Types

Request and response types are exported from the crate root, for example
`SendEmailParams`, `SearchInboxResponse`, `OutboundEmail`,
`OutboundEmailDetail`, and `TotpOtpResponse`. JSON fields named `type` are
exposed as `kind`. Status enums include an `Unknown` variant so that values
added to the API later do not break deserialization.
