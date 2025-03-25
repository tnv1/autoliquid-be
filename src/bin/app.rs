use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        // Filter events based on the RUST_LOG environment variable
        // or fall back to a default level like "info"
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,autoliquid_be=debug,indexer=debug")),
        )
        // Format the output with timestamps and colors
        .with(fmt::layer().with_target(true).with_file(true).with_line_number(true))
        .init();

    tracing::info!("Starting app");
}
