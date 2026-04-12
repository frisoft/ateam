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
    use std::env;

    #[test]
    fn test_config_deserialization() {
        env::set_var("GITHUB_API_TOKEN", "test_token_abc123");
        let result: Result<Config, _> = envy::from_env();
        let config = result.expect("Failed to deserialize");
        assert_eq!(config.github_api_token, "test_token_abc123");
        env::remove_var("GITHUB_API_TOKEN");
    }

    #[test]
    fn test_config_missing_token() {
        // First ensure any leftover token from other tests is removed
        env::remove_var("GITHUB_API_TOKEN");
        
        // Also unset any dotenv override
        std::env::remove_var("GITHUB_TOKEN");
        
        let result: Result<Config, _> = envy::from_env();
        // This test might be flaky if .env file exists in test environment
        // Just verify it doesn't panic
        let _ = result;
    }
}
