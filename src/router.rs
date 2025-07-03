use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};
use tokio::sync::RwLock;

use crate::exporter::Exporter;

// just return 200 when requesting
pub async fn heartbeat() -> StatusCode {
    StatusCode::OK
}

// route for prometheus endpoint
pub async fn metrics(state: State<Arc<RwLock<Exporter>>>) -> Response {
    let metrics;

    {
        let mut exporter = state.0.write().await;
        metrics = exporter.collect().await;
    }

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    if let Ok(m) = metrics
        && encoder.encode(&m, &mut buffer).is_ok()
    {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", encoder.format_type())
            .body(buffer.into())
            .unwrap()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
