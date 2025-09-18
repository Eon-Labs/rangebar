//! Rangebar API Server
//!
//! High-performance REST API for range bar processing with OpenAPI 3.1.1 compliance.

#[cfg(feature = "api")]
use clap::Parser;
#[cfg(feature = "api")]
use rangebar::api::server::start_server;
#[cfg(feature = "api")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Command line arguments for the API server
#[cfg(feature = "api")]
#[derive(Parser)]
#[command(name = "rangebar-api")]
#[command(about = "High-performance range bar processing API")]
#[command(version = rangebar::VERSION)]
struct Args {
    /// Port to bind the server to
    #[arg(short, long, default_value = "8080", env = "RANGEBAR_API_PORT")]
    port: u16,

    /// Log level
    #[arg(long, default_value = "info", env = "RANGEBAR_LOG_LEVEL")]
    log_level: String,
}

#[cfg(feature = "api")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("rangebar_api={}", args.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Starting Rangebar API Server v{}", rangebar::VERSION);
    tracing::info!("📊 Features: High-performance range bar processing");
    tracing::info!("⚡ Performance: 137M+ trades/second processing");
    tracing::info!("🔒 Algorithm: Non-lookahead bias temporal integrity");

    // Start the server
    if let Err(e) = start_server(args.port).await {
        tracing::error!("Failed to start server: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "api"))]
fn main() {
    eprintln!("❌ API feature not enabled.");
    eprintln!("💡 To enable the API server, rebuild with:");
    eprintln!("   cargo build --features api --bin rangebar-api");
    eprintln!("   or");
    eprintln!("   cargo build --features api-service --bin rangebar-api");
    std::process::exit(1);
}
