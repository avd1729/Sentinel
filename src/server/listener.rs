use tokio::net::TcpListener;
use tracing::info;

pub async fn run(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        info!("Accepted connection from {}", peer);

        tokio::spawn(async move {
            drop(socket);
        });
    }
}