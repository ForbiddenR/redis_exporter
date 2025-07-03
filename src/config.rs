use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub addr: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

impl Config {
    pub fn build() -> Result<Config> {
        Ok(envy::from_env()?)
    }

    pub fn get_url(&self) -> String {
        if self.username.is_empty() && self.password.is_empty() {
            format!("redis://{}", self.addr)
        } else {
            format!("redis://{}:{}@{}", self.username, self.password, self.addr)
        }
    }
}
