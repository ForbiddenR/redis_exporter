use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};
use tokio::sync::RwLock;

use crate::{exporter::Exporter, header::ExplicitHeader};

// just return 200 when requesting
pub async fn heartbeat() -> StatusCode {
    StatusCode::OK
}

// route for prometheus endpoint
pub async fn metrics(
    State(state): State<Arc<RwLock<Exporter>>>,
    header: ExplicitHeader,
) -> Response {
    let metrics = { state.write().await.collect(&header.0).await };

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    match encoder.encode(&metrics, &mut buffer) {
        Err(e) => {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        _ => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", encoder.format_type())
            .body(buffer.into())
            .unwrap(),
    }
}
