use tokio::net::TcpListener;
use tracing::info;
use crate::http::connection::Connection;
use crate::config::Config;

pub async fn run(cfg: &Config) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&cfg.server.listen_addr).await?;
    info!("Listening on {}", cfg.server.listen_addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        info!("Accepted connection from {}", peer);
        
        let static_config = cfg.static_files.clone();
        tokio::spawn(async move {
            let mut conn = Connection::new(socket, static_config);
            if let Err(e) = conn.run().await {
                tracing::error!("Connection error from {}: {}", peer, e);
            }
        });
    }
}