use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub github_api_token: String,
}

pub fn get_config() -> Result<Config, envy::Error> {
    dotenvy::dotenv().ok();
    let _ = env_logger::try_init();
    envy::from_env()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        temp_env::with_var("GITHUB_API_TOKEN", Some("test_token_abc123"), || {
            let result: Result<Config, _> = envy::from_env();
            let config = result.expect("Failed to deserialize");
            assert_eq!(config.github_api_token, "test_token_abc123");
        });
    }

    #[test]
    fn test_config_missing_token() {
        // Unset both GITHUB_API_TOKEN and any dotenv override (GITHUB_TOKEN)
        temp_env::with_vars(
            [
                ("GITHUB_API_TOKEN", None::<&str>),
                ("GITHUB_TOKEN", None::<&str>),
            ],
            || {
                let result: Result<Config, _> = envy::from_env();
                let _ = result; // Just verify it doesn't panic
            },
        );
    }
}
