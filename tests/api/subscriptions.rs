use crate::helpers::spawn_app;
use chrono::Utc;
use test_case::test_case;
use uuid::Uuid;

#[test_case(
    "name=le%20guin&email=ursula_le_guin%40gmail.com", "ursula_le_guin@gmail.com", "le guin";
    "valid name and email"
)]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(body: &str, email: &str, name: &str) {
    let app = spawn_app().await;

    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(200, response.status().as_u16(), "should return a 200");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, email, "email is not equal");
    assert_eq!(saved.name, name, "name is not equal");
}

#[test_case(
    "name=le%20guin&email=ursula_le_guin%40gmail.com", "ursula_le_guin@gmail.com", "le guin";
    "valid name and email"
)]
#[tokio::test]
#[ignore = "ignore until row exists validation is implemented"]
async fn subscribe_returns_a_400_for_existing_data_in_db(body: &str, email: &str, name: &str) {
    let app = spawn_app().await;

    let row_time_created = sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4) RETURNING subscribed_at"#,
        Uuid::new_v4(),
        email,
        name,
        Utc::now()
    )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.")
        .subscribed_at;

    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(400, response.status().as_u16(), "should return a 400");

    let agg = sqlx::query!("SELECT COUNT(*) FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(agg.count, Some(1), "there should not be any new rows");

    let exists = sqlx::query!("SELECT email, name, subscribed_at FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(exists.email, email, "email should not be updated");
    assert_eq!(exists.name, name, "name should not be updated");
    assert_eq!(
        exists.subscribed_at, row_time_created,
        "subscribed_at should not be updated"
    );
}

#[test_case(
    "name=le%20guin", "missing the email";
    "missing email"
)]
#[test_case(
    "email=ursula_le_guin%40gmail.com", "missing the name";
    "missing name"
)]
#[test_case(
    "", "missing both name and email";
    "empty body"
)]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing(invalid_body: &str, error_message: &str) {
    let app = spawn_app().await;

    let response = app.post_subscriptions(invalid_body.into()).await;
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );

    let agg = sqlx::query!("SELECT COUNT(*) FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(agg.count, Some(0), "there should not be any new rows");
}

#[test_case(
    "name=&email=ursula_le_guin%40gmail.com", "empty name";
    "empty name"
)]
#[test_case(
    "name=Ursula&email=", "empty email";
    "empty email"
)]
#[test_case(
    "name=&email=", "empty name and email";
    "empty name and email"
)]
#[test_case(
    "name=Ursula&email=definitely-not-an-email", "invalid email";
    "invalid email"
)]
#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid(
    invalid_body: &str,
    error_message: &str,
) {
    let app = spawn_app().await;

    let response = app.post_subscriptions(invalid_body.into()).await;
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );

    let agg = sqlx::query!("SELECT COUNT(*) FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(agg.count, Some(0), "there should not be any new rows");
}
