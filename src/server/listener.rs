use crate::config::Config;
use crate::http::connection::Connection;
use crate::proxy::{BackendPool, ProxyHandler};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::info;

pub async fn run(cfg: &Config) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&cfg.server.listen_addr).await?;
    info!("Listening on {}", cfg.server.listen_addr);

    // Initialize proxy handler if configured
    let proxy_handler = if let Some(ref proxy_config) = cfg.proxy {
        // Validate backend configuration
        proxy_config.validate()?;

        // Create backend pool
        let pool = BackendPool::new(proxy_config.backends.clone());
        
        info!(
            backends = proxy_config.backends.len(),
            "Initialized backend pool"
        );

        // Create proxy handler
        let handler = ProxyHandler::new(
            pool,
            Duration::from_millis(proxy_config.connection_timeout_ms),
            Duration::from_millis(proxy_config.request_timeout_ms),
        );

        Some(Arc::new(handler))
    } else {
        info!("No proxy configuration found, serving static files only");
        None
    };

    loop {
        let (socket, peer) = listener.accept().await?;
        info!("Accepted connection from {}", peer);

        let static_config = cfg.static_files.clone();
        let proxy = proxy_handler.clone();

        tokio::spawn(async move {
            let mut conn = if let Some(proxy_handler) = proxy {
                Connection::with_proxy(socket, static_config, proxy_handler)
            } else {
                Connection::new(socket, static_config)
            };

            if let Err(e) = conn.run().await {
                tracing::error!("Connection error from {}: {}", peer, e);
            }
        });
    }
}
