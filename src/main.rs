use std::{error::Error, sync::Arc};

use axum::{Router, routing::get};
use redis_exporter::{config::Config, exporter::Exporter, router};
use tokio::{net::TcpListener, sync::RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let c = Config::build()?;

    let client = redis::Client::open(c.get_url())?;

    let exporter = Exporter::new(client);

    let state = Arc::new(RwLock::new(exporter));

    let app = Router::new()
        .route("/metrics", get(router::metrics))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
