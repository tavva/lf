use anyhow::Result;
use clap::Subcommand;

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output};
use crate::types::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum SessionsCommands {
    /// List sessions with optional filters
    List {
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

    /// Show details of a specific session
    Show {
        /// Session ID
        id: String,

        /// Include associated traces
        #[arg(long)]
        with_traces: bool,

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

impl SessionsCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            SessionsCommands::List {
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

                let sessions = client
                    .list_sessions(from.as_deref(), to.as_deref(), *limit, *page)
                    .await?;

                format_and_output(
                    &sessions,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            SessionsCommands::Show {
                id,
                with_traces,
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

                let mut session = client.get_session(id).await?;

                // Fetch traces if requested
                if *with_traces {
                    let traces = client
                        .list_traces(
                            None,
                            None,
                            Some(id),
                            None,
                            None,
                            None,
                            100,
                            1,
                        )
                        .await?;
                    session.traces = traces;
                }

                format_and_output(
                    &session,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }
        }
    }
}
