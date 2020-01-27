use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub github_api_token: String,
}

pub fn get_config() -> Result<Config, envy::Error> {
    dotenv::dotenv().ok();
    env_logger::init();
    envy::from_env()
}
