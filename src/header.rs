use std::str::FromStr;

use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, request::Parts},
};

use crate::info::Mode;

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standard" => Ok(Mode::Standard),
            "simple" => Ok(Mode::Simple),
            _ => Err("invalid mode, must be 'standard' or 'simple'"),
        }
    }
}

impl TryFrom<&HeaderValue> for Mode {
    type Error = &'static str;

    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        Ok(value
            .to_str()
            .map_err(|_| "invalid header value")?
            .parse()?)
    }
}

#[derive(Debug)]
pub struct ExplicitHeader(pub Option<Mode>);

impl<S> FromRequestParts<S> for ExplicitHeader
where
    S: Send + Sync,
{
    type Rejection = &'static str;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(ExplicitHeader(
            parts
                .extract::<HeaderMap>()
                .await
                .map_err(|_| "extract headers failed")?
                .get("EXPORTER_MODE")
                .map(|h| h.try_into())
                .transpose()?,
        ))
    }
}
