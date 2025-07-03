use std::{error::Error, fmt::Display, num::ParseFloatError, result};

pub type Result<T> = result::Result<T, MyError>;

#[derive(Debug)]
pub enum MyError {
    Redis(redis::RedisError),
    ParseFloatErr(ParseFloatError),
    Prometheus(prometheus::Error),
    Config(envy::Error),
    SplitFail(String),
}

impl Error for MyError {}

impl From<redis::RedisError> for MyError {
    fn from(e: redis::RedisError) -> Self {
        Self::Redis(e)
    }
}

impl From<ParseFloatError> for MyError {
    fn from(e: ParseFloatError) -> Self {
        Self::ParseFloatErr(e)
    }
}

impl From<prometheus::Error> for MyError {
    fn from(e: prometheus::Error) -> Self {
        Self::Prometheus(e)
    }
}

impl From<envy::Error> for MyError {
    fn from(e: envy::Error) -> Self {
        Self::Config(e)
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redis(e) => write!(f, "redis error: {e}"),
            Self::ParseFloatErr(e) => write!(f, "parse float error: {e}"),
            Self::Prometheus(e) => write!(f, "prometheus error: {e}"),
            Self::SplitFail(e) => write!(f, "split error: {e}"),
            Self::Config(e) => write!(f, "config error: {e}"),
        }
    }
}
