use chrono::Utc;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use test_case::test_case;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success(), "status should be success");
    assert_eq!(
        Some(0),
        response.content_length(),
        "content should be empty"
    );
}

#[test_case(
    "name=le%20guin&email=ursula_le_guin%40gmail.com", "ursula_le_guin@gmail.com", "le guin";
    "valid name and email"
)]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(body: &str, email: &str, name: &str) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
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
async fn subscribe_returns_a_400_for_existing_data_in_db(body: &str, email: &str, name: &str) {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

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

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
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
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
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
