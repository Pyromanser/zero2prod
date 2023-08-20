use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};
use serde_json::Value;
use test_case::test_case;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

/// Use the public API of the application under test to create
/// an confirmed subscriber.
async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    });
    let response = app.post_newsletters(newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[test_case(
    serde_json::json!({}), "empty body";
    "empty body"
)]
#[test_case(
    serde_json::json!({
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>",
        }
    }), "missing title";
    "missing title"
)]
#[test_case(
    serde_json::json!({"title": "Newsletter!"}), "missing content";
    "missing content"
)]
#[test_case(
    serde_json::json!({
        "title": "Newsletter!",
        "content": {}
    }), "empty content";
    "empty content"
)]
#[test_case(
    serde_json::json!({
        "title": "Newsletter!",
        "content": {
            "html": "<p>Newsletter body as HTML</p>",
    }
    }), "missing content's text";
    "missing content's text"
)]
#[test_case(
    serde_json::json!({
        "title": "Newsletter!",
        "content": {
            "text": "Newsletter body as plain text",
    }
    }), "missing content's html";
    "missing content's html"
)]
#[tokio::test]
async fn newsletters_returns_400_for_invalid_data(invalid_body: Value, error_message: &str) {
    let app = spawn_app().await;
    let response = app.post_newsletters(invalid_body).await;
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );
}