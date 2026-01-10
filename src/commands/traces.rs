use anyhow::Result;
use clap::Subcommand;

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output};
use crate::types::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum TracesCommands {
    /// List traces with optional filters
    List {
        /// Filter by trace name
        #[arg(short, long)]
        name: Option<String>,

        /// Filter by user ID
        #[arg(short, long)]
        user_id: Option<String>,

        /// Filter by session ID
        #[arg(short, long)]
        session_id: Option<String>,

        /// Filter by tags (can be specified multiple times)
        #[arg(short, long)]
        tags: Option<Vec<String>>,

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

    /// Get a specific trace by ID
    Get {
        /// Trace ID
        id: String,

        /// Include observations
        #[arg(long)]
        with_observations: bool,

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

impl TracesCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            TracesCommands::List {
                name,
                user_id,
                session_id,
                tags,
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

                let traces = client
                    .list_traces(
                        name.as_deref(),
                        user_id.as_deref(),
                        session_id.as_deref(),
                        tags.as_deref(),
                        from.as_deref(),
                        to.as_deref(),
                        *limit,
                        *page,
                    )
                    .await?;

                format_and_output(
                    &traces,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            TracesCommands::Get {
                id,
                with_observations,
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

                let mut trace = client.get_trace(id).await?;

                // Fetch observations if requested
                if *with_observations {
                    let observations = client
                        .list_observations(
                            Some(id),
                            None,
                            None,
                            None,
                            None,
                            None,
                            100,
                            1,
                        )
                        .await?;
                    trace.observations = observations
                        .into_iter()
                        .map(|o| serde_json::to_value(o).unwrap_or_default())
                        .collect();
                }

                format_and_output(
                    &trace,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }
        }
    }
}
