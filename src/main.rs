mod config;
mod server;
mod http;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cfg = Config::load();

    tokio::select! {
        res = server::listener::run(&cfg.listen_addr) => {
            res?;
        }

        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
    }

    Ok(())
}
