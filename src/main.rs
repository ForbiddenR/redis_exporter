use std::{error::Error, sync::Arc};

use axum::{Router, routing::get};
use cfg_if::cfg_if;
use redis_exporter::{config::Config, exporter::Exporter, router};
use tokio::{net::TcpListener, sync::RwLock};

cfg_if! {
    if #[cfg(feature = "mi-malloc")] {
        use mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        // health checker
        .route("/heartbeat", get(router::heartbeat))
        // prometheus' endpoint
        .route("/metrics", get(router::metrics))
        // add exporter which will collect prometheus' metrics
        .with_state(Arc::new(RwLock::new(Exporter::new(redis::Client::open(
            Config::build()?.get_url(),
        )?))));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
