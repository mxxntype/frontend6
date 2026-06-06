use clap::Parser;
use color_eyre::eyre::Report;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup_diagnostics()?;

    let settings = f6_server::Settings::parse();
    f6_server::run(settings).await?;

    Ok(())
}

fn setup_diagnostics() -> Result<(), Report> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{EnvFilter, fmt};

    color_eyre::install()?;

    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    let format_layer = fmt::layer().without_time().with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .with(ErrorLayer::default())
        .try_init()?;

    Ok(())
}
