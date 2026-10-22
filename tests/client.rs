use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mailisk::*;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

async fn setup() -> (MockServer, MailiskClient) {
    let server = MockServer::start().await;
    let client = MailiskClient::builder("test-key")
        .base_url(server.uri())
        .build()
        .unwrap();
    (server, client)
}

async fn only_request(server: &MockServer) -> Request {
    let mut requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1, "expected exactly one request");
    requests.remove(0)
}

fn query(request: &Request) -> Vec<(String, String)> {
    request.url.query_pairs().into_owned().collect()
}

fn query_value(request: &Request, name: &str) -> Option<String> {
    query(request)
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}

fn now_unix() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

fn email_json(id: &str) -> Value {
    json!({
        "id": id,
        "from": {"address": "sender@example.com"},
        "to": [{"address": "john@ns.mailisk.net", "name": "John"}],
        "subject": "Hello",
        "text": "Body",
        "received_date": "2026-05-11T18:00:00.000Z",
        "received_timestamp": 1778522400,
        "expires_timestamp": 1778608800,
        "attachments": [{"id": "att-1", "filename": "a.txt", "contentType": "text/plain", "size": 3}]
    })
}

fn outbound_json(id: &str, kind: &str) -> Value {
    json!({
        "id": id,
        "organisation_id": "org-1",
        "type": kind,
        "status": "queued",
        "from": {"email": "from@ns.mailisk.net", "name": "From"},
        "subject": "Subject",
        "recipient_count": 1,
        "attachment_count": 0,
        "message_id": "<message@mailisk>",
        "queued_at": "2026-05-11T18:00:00.000Z",
        "created_at": "2026-05-11T18:00:00.000Z",
        "updated_at": "2026-05-11T18:00:00.000Z"
    })
}

fn totp_device_json() -> Value {
    json!({
        "id": "device-id",
        "organisation_id": "org-1",
        "name": "Device",
        "username": null,
        "issuer": null,
        "digits": 6,
        "period": 30,
        "algorithm": "SHA1",
        "source": "shared_secret",
        "expires_at": null,
        "created_at": "2026-05-11T18:00:00.000Z",
        "updated_at": "2026-05-11T18:00:00.000Z"
    })
}

#[test]
fn builder_validates_configuration() {
    let client = MailiskClient::new("test-key").unwrap();
    assert_eq!(client.base_url().as_str(), "https://api.mailisk.com/");

    assert!(matches!(MailiskClient::new(""), Err(Error::Config(_))));
    assert!(matches!(
        MailiskClient::builder("test-key").base_url("not a url").build(),
        Err(Error::Config(_))
    ));
}

#[tokio::test]
async fn list_namespaces_sends_api_key() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/namespaces"))
        .and(header("X-Api-Key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "data": [{"id": "id", "namespace": "ns"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client.list_namespaces().await.unwrap();

    assert_eq!(result.total_count, 1);
    assert_eq!(result.data[0].namespace, "ns");
    assert_eq!(result.data[0].expires_at, None);
    assert!(only_request(&server).await.url.query().is_none());
}

#[tokio::test]
async fn base_url_path_prefix_is_kept() {
    let server = MockServer::start().await;
    let client = MailiskClient::builder("test-key")
        .base_url(format!("{}/prefix", server.uri()))
        .build()
        .unwrap();
    Mock::given(path("/prefix/api/namespaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"total_count": 0, "data": []})))
        .expect(1)
        .mount(&server)
        .await;

    client.list_namespaces().await.unwrap();
}

#[tokio::test]
async fn search_inbox_uses_client_defaults() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/emails/test-namespace/inbox"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "options": {"limit": 20, "offset": 0, "wait": true},
            "data": [email_json("email-1")]
        })))
        .mount(&server)
        .await;

    let before = now_unix() - 15 * 60 - 2;
    let result = client
        .search_inbox("test-namespace", &SearchInboxParams::default())
        .await
        .unwrap();
    let after = now_unix() - 15 * 60 + 2;

    let request = only_request(&server).await;
    assert_eq!(query_value(&request, "wait").as_deref(), Some("true"));
    let from_timestamp: i64 = query_value(&request, "from_timestamp").unwrap().parse().unwrap();
    assert!((before..=after).contains(&from_timestamp));
    assert_eq!(query(&request).len(), 2);

    let email = &result.data[0];
    assert_eq!(email.from.address, "sender@example.com");
    assert_eq!(email.to[0].name.as_deref(), Some("John"));
    assert!(email.cc.is_empty());
    assert_eq!(email.attachments[0].content_type.as_deref(), Some("text/plain"));
    assert_eq!(result.options.wait, Some(true));
}

#[tokio::test]
async fn search_inbox_accepts_custom_params() {
    let (server, client) = setup().await;
    Mock::given(path("/api/emails/test-namespace/inbox"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 0,
            "options": {"limit": 5, "offset": 10, "wait": false},
            "data": []
        })))
        .mount(&server)
        .await;

    client
        .search_inbox_with_options(
            "test-namespace",
            &SearchInboxParams {
                limit: Some(5),
                offset: Some(10),
                from_timestamp: Some(1_700_000_000),
                to_addr_prefix: Some("john@ns.mailisk.net".into()),
                subject_includes: Some("password reset".into()),
                wait: Some(false),
                ..Default::default()
            },
            &RequestOptions::timeout(Duration::from_secs(10)),
        )
        .await
        .unwrap();

    let mut pairs = query(&only_request(&server).await);
    pairs.sort();
    let expected: Vec<(String, String)> = [
        ("from_timestamp", "1700000000"),
        ("limit", "5"),
        ("offset", "10"),
        ("subject_includes", "password reset"),
        ("to_addr_prefix", "john@ns.mailisk.net"),
        ("wait", "false"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect();
    assert_eq!(pairs, expected);
}

#[tokio::test]
async fn waiting_search_follows_redirects_with_api_key() {
    let (server, client) = setup().await;
    let inbox_path = "/api/emails/test-namespace/inbox";
    Mock::given(path(inbox_path))
        .and(query_param("wait", "true"))
        .respond_with(|request: &Request| {
            let location = format!("{}?{}", request.url.path(), request.url.query().unwrap_or_default());
            ResponseTemplate::new(302).insert_header("Location", location.as_str())
        })
        .up_to_n_times(3)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(path(inbox_path))
        .and(header("X-Api-Key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "options": {"limit": 20, "offset": 0},
            "data": [email_json("email-1")]
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let result = client
        .search_inbox("test-namespace", &SearchInboxParams::default())
        .await
        .unwrap();

    assert_eq!(result.data[0].id, "email-1");
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 4);
    assert!(requests.iter().all(|request| request.headers.contains_key("x-api-key")));
}

#[tokio::test]
async fn search_sms_messages_uses_client_defaults() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/sms/+15550000001/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "options": {"limit": 20, "offset": 0, "direction": "inbound", "wait": true},
            "data": [{
                "id": "sms-1",
                "sms_phone_number_id": "number-1",
                "body": "Code 1234",
                "from_number": "+15550000002",
                "to_number": "+15550000001",
                "provider_message_id": null,
                "created_at": "2026-05-11T18:00:00.000Z",
                "direction": "inbound"
            }]
        })))
        .mount(&server)
        .await;

    let result = client
        .search_sms_messages(
            "+15550000001",
            &SearchSmsMessagesParams {
                direction: Some(SmsMessageDirection::Inbound),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let request = only_request(&server).await;
    assert_eq!(query_value(&request, "wait").as_deref(), Some("true"));
    assert_eq!(query_value(&request, "direction").as_deref(), Some("inbound"));
    let from_date = query_value(&request, "from_date").unwrap();
    assert_eq!(from_date.len(), "2026-05-11T18:00:00.000Z".len());
    assert!(from_date.ends_with('Z'));
    assert_eq!(result.data[0].direction, SmsMessageDirection::Inbound);
    assert_eq!(result.options.direction, Some(SmsMessageDirection::Inbound));
}

#[tokio::test]
async fn outbound_email_methods() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/api/emails/send"))
        .and(query_param("namespace", "test-namespace"))
        .and(body_json(json!({
            "subject": "Hello",
            "from": {"email": "from@ns.mailisk.net", "name": "From"},
            "to": ["to@ns.mailisk.net"],
            "text": "Body",
            "attachments": [{
                "filename": "a.txt",
                "content_type": "text/plain",
                "content_base64": "YWJj",
                "disposition": "inline"
            }]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(outbound_json("out-1", "new")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/emails/outbound/out-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut detail = outbound_json("out-1", "new");
            detail["delivery_summary"] = json!({"queued": 1});
            detail["recipients"] = json!([{
                "id": "rcpt-1",
                "type": "to",
                "email": "to@ns.mailisk.net",
                "delivery_status": "queued",
                "attempt_count": 0
            }]);
            detail["events"] = json!([{
                "id": "event-1",
                "provider": "ses",
                "event_type": "some_future_event",
                "occurred_at": "2026-05-11T18:00:00.000Z",
                "recipients": [],
                "created_at": "2026-05-11T18:00:00.000Z"
            }]);
            detail
        }))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/emails/email-1/reply"))
        .and(body_json(json!({"text": "Reply"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(outbound_json("out-2", "reply")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/emails/email-1/forward"))
        .and(body_json(json!({"to": ["fwd@ns.mailisk.net"], "subject": "Fwd"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(outbound_json("out-3", "forward")))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .send_email(
            "test-namespace",
            &SendEmailParams {
                subject: "Hello".into(),
                from: Some(OutboundEmailAddress::with_name("from@ns.mailisk.net", "From")),
                to: vec!["to@ns.mailisk.net".into()],
                text: Some("Body".into()),
                attachments: vec![OutboundEmailAttachment {
                    filename: "a.txt".into(),
                    content_type: "text/plain".into(),
                    content_base64: "YWJj".into(),
                    content_id: None,
                    disposition: Some(OutboundEmailAttachmentDisposition::Inline),
                }],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(sent.kind, OutboundEmailType::New);
    assert_eq!(sent.status, OutboundEmailStatus::Queued);

    let detail = client.get_outbound_email("out-1").await.unwrap();
    assert_eq!(detail.id, "out-1");
    assert_eq!(detail.delivery_summary.queued, 1);
    assert_eq!(detail.recipients[0].kind, OutboundEmailRecipientType::To);
    assert_eq!(detail.events[0].event_type, OutboundEmailEventType::Unknown);

    let reply = client
        .reply_to_email(
            "email-1",
            &ReplyToEmailParams {
                text: Some("Reply".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(reply.kind, OutboundEmailType::Reply);

    let forward = client
        .forward_email(
            "email-1",
            &ForwardEmailParams {
                to: vec!["fwd@ns.mailisk.net".into()],
                subject: Some("Fwd".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(forward.kind, OutboundEmailType::Forward);
}

#[tokio::test]
async fn sms_methods() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/sms/numbers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "data": [{
                "id": "number-1",
                "organisation_id": "org-1",
                "status": "active",
                "country": "US",
                "phone_number": "+15550000001",
                "created_at": "2026-05-11T18:00:00.000Z",
                "updated_at": "2026-05-11T18:00:00.000Z"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/sms/virtual"))
        .and(body_json(json!({
            "from_number": "+15550000002",
            "to_number": "+15550000001",
            "body": "Hello"
        })))
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;

    let numbers = client.list_sms_numbers().await.unwrap();
    assert_eq!(numbers.data[0].status, SmsNumberStatus::Active);

    client
        .send_virtual_sms(&SendVirtualSmsParams {
            from_number: "+15550000002".into(),
            to_number: "+15550000001".into(),
            body: "Hello".into(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn totp_methods() {
    let (server, client) = setup().await;
    let device = || ResponseTemplate::new(200).set_body_json(totp_device_json());
    let otp = || {
        ResponseTemplate::new(200).set_body_json(json!({
            "code": "123456",
            "expires": "2026-05-11T18:00:30.000Z"
        }))
    };

    Mock::given(method("GET"))
        .and(path("/api/devices"))
        .and(query_param("username", "john"))
        .and(query_param("issuer", "Acme"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "total_count": 1,
            "options": {"limit": 20, "offset": 0, "username": "john", "issuer": "Acme"},
            "items": [totp_device_json()]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/devices"))
        .and(body_json(
            json!({"shared_secret": "JBSWY3DPEHPK3PXP", "name": "Device"}),
        ))
        .respond_with(device())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/devices/custom"))
        .and(body_json(
            json!({"secret": "secret", "digits": 8, "algorithm": "SHA256"}),
        ))
        .respond_with(device())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/devices/base32-secret-key"))
        .and(body_json(
            json!({"base32_secret_key": "JBSWY3DPEHPK3PXP", "period": 60}),
        ))
        .respond_with(device())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/devices/otpauth-url"))
        .and(body_json(
            json!({"otp_auth_url": "otpauth://totp/Acme:john?secret=JBSWY3DPEHPK3PXP"}),
        ))
        .respond_with(device())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/devices/otp"))
        .and(body_json(
            json!({"shared_secret": "JBSWY3DPEHPK3PXP", "min_seconds_until_expire": 10}),
        ))
        .respond_with(otp())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/devices/device-id/otp"))
        .and(query_param("min_seconds_until_expire", "10"))
        .respond_with(otp())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/api/devices/device-id"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let listed = client
        .list_totp_devices(&ListTotpDevicesParams {
            username: Some("  john ".into()),
            issuer: Some(" Acme".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(listed.items[0].algorithm, TotpAlgorithm::Sha1);

    client
        .create_totp_device(&CreateTotpDeviceParams {
            shared_secret: "JBSWY3DPEHPK3PXP".into(),
            name: Some("Device".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    client
        .create_custom_totp_device(&CreateCustomTotpDeviceParams {
            secret: "secret".into(),
            options: TotpDeviceOptions {
                digits: Some(8),
                algorithm: Some(TotpAlgorithm::Sha256),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .create_totp_device_from_base32_secret_key(&CreateBase32SecretKeyTotpDeviceParams {
            base32_secret_key: "JBSWY3DPEHPK3PXP".into(),
            options: TotpDeviceOptions {
                period: Some(60),
                ..Default::default()
            },
        })
        .await
        .unwrap();
    client
        .create_totp_device_from_otp_auth_url(&CreateOtpAuthUrlTotpDeviceParams {
            otp_auth_url: "otpauth://totp/Acme:john?secret=JBSWY3DPEHPK3PXP".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    let min_expiry = GetTotpOtpParams {
        min_seconds_until_expire: Some(10),
    };
    let by_secret = client
        .get_totp_otp_by_shared_secret("JBSWY3DPEHPK3PXP", &min_expiry)
        .await
        .unwrap();
    assert_eq!(by_secret.code, "123456");
    client
        .get_totp_otp_by_device_id("device-id", &min_expiry)
        .await
        .unwrap();
    client.delete_totp_device("device-id").await.unwrap();
}

#[tokio::test]
async fn download_attachment_uses_presigned_url_without_api_key() {
    let (server, client) = setup().await;
    Mock::given(path("/api/attachments/attachment-id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "id": "attachment-id",
                "filename": "report.txt",
                "content_type": "text/plain",
                "size": 11,
                "expires_at": null,
                "download_url": format!("{}/downloads/report.txt?signature=abc", server.uri())
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/downloads/report.txt"))
        .and(query_param("signature", "abc"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello world".to_vec()))
        .expect(1)
        .mount(&server)
        .await;

    let content = client
        .download_attachment_with_options("attachment-id", &RequestOptions::timeout(Duration::from_secs(5)))
        .await
        .unwrap();

    assert_eq!(content, b"hello world");
    let requests = server.received_requests().await.unwrap();
    assert!(requests[0].headers.contains_key("x-api-key"));
    assert!(!requests[1].headers.contains_key("x-api-key"));
}

#[tokio::test]
async fn api_errors_include_status_and_body() {
    let (server, client) = setup().await;
    Mock::given(header_exists("X-Api-Key"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "statusCode": 401,
            "message": "Invalid API key"
        })))
        .mount(&server)
        .await;

    let error = client.list_namespaces().await.unwrap_err();

    assert_eq!(error.status().map(|status| status.as_u16()), Some(401));
    match error {
        Error::Api { body, .. } => assert!(body.contains("Invalid API key")),
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
