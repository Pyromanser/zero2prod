use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    #[allow(clippy::must_use_candidate)]
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    #[allow(clippy::must_use_candidate)]
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

/// # Errors
///
/// Returns an error if any of the required environment variables are missing or if the configuration file contains invalid values.
#[allow(clippy::module_name_repetitions)]
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?
        .try_deserialize()
}
