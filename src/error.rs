use std::{error::Error, fmt::Display, num::ParseFloatError, result};

pub type Result<T> = result::Result<T, MyError>;

#[derive(Debug)]
pub enum MyError {
    Redis(redis::RedisError),
    ParseFloat(ParseFloatError),
    Prometheus(prometheus::Error),
    Config(envy::Error),
    Split(String),
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Redis(e) => Some(e),
            Self::ParseFloat(e) => Some(e),
            Self::Prometheus(e) => Some(e),
            Self::Config(e) => Some(e),
            Self::Split(_) => None,
        }
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redis(e) => write!(f, "redis error: {e}"),
            Self::ParseFloat(e) => write!(f, "parse float error: {e}"),
            Self::Prometheus(e) => write!(f, "prometheus error: {e}"),
            Self::Split(e) => write!(f, "split error: {e}"),
            Self::Config(e) => write!(f, "config error: {e}"),
        }
    }
}

macro_rules! impl_from {
    ($($var:ident($type:ty)),+ $(,)?) => {
        $(
            impl From<$type> for MyError {
                fn from(e: $type) -> Self {
                    Self::$var(e)
                }
            }
        )+
    };
}

impl_from! {
    Redis(redis::RedisError),
    Prometheus(prometheus::Error),
    Config(envy::Error),
    ParseFloat(ParseFloatError),
    Split(String),
}
