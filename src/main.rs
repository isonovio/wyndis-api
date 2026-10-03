use std::net::SocketAddr;

use anyhow::Result;
use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;

use wyndis_api::{args::Args, config::Config, env::Env, logging, routes};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let env = Env::load()?;
    logging::init(env.log_filter.clone());
    let config = Config::load(args.config)?;

    let addr = config.bind_addr;
    let router = routes::init(config, &env)?;
    serve(addr, router).await?;

    Ok(())
}

async fn serve(addr: SocketAddr, router: Router) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(address = %listener.local_addr()?, "listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to listen for shutdown signal");
    }
}
