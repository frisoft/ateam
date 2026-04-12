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

    struct EnvGuard {
        key: String,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            env::remove_var(&self.key);
        }
    }

    fn set_env(key: &str, value: &str) -> EnvGuard {
        env::set_var(key, value);
        EnvGuard { key: key.to_string() }
    }

    #[test]
    fn test_config_deserialization() {
        // Test that Config can be deserialized from environment
        env::set_var("GITHUB_API_TOKEN", "test_token_abc123");
        let result: Result<Config, _> = envy::from_env();
        let config = result.expect("Failed to deserialize");
        assert_eq!(config.github_api_token, "test_token_abc123");
        env::remove_var("GITHUB_API_TOKEN");
    }

    #[test]
    fn test_config_missing_token() {
        // Ensure the env var is not set
        env::remove_var("GITHUB_API_TOKEN");
        
        let result: Result<Config, _> = envy::from_env();
        assert!(result.is_err());
    }
}
