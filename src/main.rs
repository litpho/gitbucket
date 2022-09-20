//! Gitbucket is a commandline tool to make it easy to mirror Bitbucket locally

mod cli;

use std::env;
use std::str::FromStr;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// The asynchronous (Tokio) main method
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let start = std::time::Instant::now();
    let _ansi_support = ansi_term::enable_ansi_support();

    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| String::from("INFO"));
    let filter = EnvFilter::from_str(&log_level)?;
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(filter)
        .init();

    let command = cli::SubCommand::from_arguments()?;
    match command {
        cli::SubCommand::Clone {
            git,
            bitbucket_root_url,
            credentials,
            exclusions,
        } => {
            git.clone_command(&bitbucket_root_url, &credentials, exclusions)
                .await?
        }
        cli::SubCommand::Featured { git, show_main } => git.featured_command(show_main).await?,
        cli::SubCommand::Pull { git, show_errors } => git.pull_command(show_errors).await?,
        cli::SubCommand::Status { git } => git.status_command().await?,
    }

    info!("Finished in {}ms", start.elapsed().as_millis());

    Ok(())
}
