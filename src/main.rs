use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;
mod config;
mod formatters;
mod types;

use commands::config::ConfigCommands;
use commands::datasets::DatasetsCommands;
use commands::metrics::MetricsCommands;
use commands::observations::ObservationsCommands;
use commands::prompts::PromptsCommands;
use commands::scores::ScoresCommands;
use commands::sessions::SessionsCommands;
use commands::traces::TracesCommands;

/// Langfuse CLI - Command-line interface for Langfuse observability platform
#[derive(Parser)]
#[command(name = "lf")]
#[command(author = "Langfuse CLI Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Command-line interface for Langfuse LLM observability platform", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage configuration profiles
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Query and manage traces
    #[command(subcommand)]
    Traces(TracesCommands),

    /// Query and manage sessions
    #[command(subcommand)]
    Sessions(SessionsCommands),

    /// Query and manage observations
    #[command(subcommand)]
    Observations(ObservationsCommands),

    /// Query and manage scores
    #[command(subcommand)]
    Scores(ScoresCommands),

    /// Query metrics with aggregations
    #[command(subcommand)]
    Metrics(MetricsCommands),

    /// Manage prompts
    #[command(subcommand)]
    Prompts(PromptsCommands),

    /// Manage datasets for evaluation
    #[command(subcommand)]
    Datasets(DatasetsCommands),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    match cli.command {
        Commands::Config(cmd) => cmd.execute().await,
        Commands::Traces(cmd) => cmd.execute().await,
        Commands::Sessions(cmd) => cmd.execute().await,
        Commands::Observations(cmd) => cmd.execute().await,
        Commands::Scores(cmd) => cmd.execute().await,
        Commands::Metrics(cmd) => cmd.execute().await,
        Commands::Prompts(cmd) => cmd.execute().await,
        Commands::Datasets(cmd) => cmd.execute().await,
    }
}
