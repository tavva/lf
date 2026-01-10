use anyhow::Result;
use clap::Subcommand;

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output};
use crate::types::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum ScoresCommands {
    /// List scores with optional filters
    List {
        /// Filter by score name
        #[arg(short, long)]
        name: Option<String>,

        /// Filter from timestamp (ISO 8601 format)
        #[arg(long)]
        from: Option<String>,

        /// Filter to timestamp (ISO 8601 format)
        #[arg(long)]
        to: Option<String>,

        /// Maximum number of results
        #[arg(short, long, default_value = "50")]
        limit: u32,

        /// Page number
        #[arg(short, long, default_value = "1")]
        page: u32,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get a specific score by ID
    Get {
        /// Score ID
        id: String,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

impl ScoresCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            ScoresCommands::List {
                name,
                from,
                to,
                limit,
                page,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    Some(*limit),
                    Some(*page),
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                let scores = client
                    .list_scores(
                        name.as_deref(),
                        from.as_deref(),
                        to.as_deref(),
                        *limit,
                        *page,
                    )
                    .await?;

                format_and_output(
                    &scores,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            ScoresCommands::Get {
                id,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    None,
                    None,
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                let score = client.get_score(id).await?;

                format_and_output(
                    &score,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }
        }
    }
}
