mod config;
mod http;
mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cfg = Config::load();

    tokio::select! {
        res = server::listener::run(&cfg) => {
            res?;
        }

        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
    }

    Ok(())
}
