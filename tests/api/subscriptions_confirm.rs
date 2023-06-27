use crate::helpers::spawn_app;
use test_case::test_case;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[test_case(
    "name=le%20guin&email=ursula_le_guin%40gmail.com";
    "valid name and email"
)]
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(body: &str) {
    let app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(response.status().as_u16(), 200, "should return a 200");
}

#[test_case(
    "name=le%20guin&email=ursula_le_guin%40gmail.com", "ursula_le_guin@gmail.com", "le guin";
    "valid name and email"
)]
#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber(
    body: &str,
    email: &str,
    name: &str,
) {
    let app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, email, "email is not equal");
    assert_eq!(saved.name, name, "name is not equal");
    assert_eq!(saved.status, "confirmed");
}
