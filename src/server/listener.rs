use tokio::net::TcpListener;
use tracing::info;
use crate::http::connection::Connection;

pub async fn run(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        info!("Accepted connection from {}", peer);

        tokio::spawn(async move {
            let mut conn = Connection::new(socket);
            if let Err(e) = conn.run().await {
                tracing::error!("Connection error from {}: {}", peer, e);
            }
        });
    }
}