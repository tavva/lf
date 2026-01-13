// ABOUTME: Command handlers for dataset management operations
// ABOUTME: Supports list, get, create for datasets, items, and runs

use anyhow::Result;
use clap::Subcommand;

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output};
use crate::types::OutputFormat;

#[derive(Debug, Subcommand)]
pub enum DatasetsCommands {
    /// List datasets
    List {
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

    /// Get a dataset by name
    Get {
        /// Dataset name
        name: String,

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

    /// Create a new dataset
    Create {
        /// Dataset name
        name: String,

        /// Dataset description
        #[arg(short, long)]
        description: Option<String>,

        /// Metadata as JSON string
        #[arg(short, long)]
        metadata: Option<String>,

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

    /// List dataset items
    Items {
        /// Filter by dataset name
        #[arg(short, long)]
        dataset: Option<String>,

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

    /// Get a dataset item by ID
    ItemGet {
        /// Item ID
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

    /// Create a dataset item
    ItemCreate {
        /// Dataset name to add item to
        #[arg(short, long)]
        dataset: String,

        /// Input data as JSON string
        #[arg(short, long)]
        input: String,

        /// Expected output as JSON string
        #[arg(short, long)]
        expected_output: Option<String>,

        /// Metadata as JSON string
        #[arg(short, long)]
        metadata: Option<String>,

        /// Source trace ID
        #[arg(long)]
        source_trace_id: Option<String>,

        /// Source observation ID
        #[arg(long)]
        source_observation_id: Option<String>,

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

    /// List runs for a dataset
    Runs {
        /// Dataset name
        dataset: String,

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

    /// Get a specific run
    RunGet {
        /// Dataset name
        dataset: String,

        /// Run name
        run: String,

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

impl DatasetsCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            DatasetsCommands::List {
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let datasets = client.list_datasets(*limit, *page).await?;

                format_and_output(
                    &datasets,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::Get {
                name,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let dataset = client.get_dataset(name).await?;

                format_and_output(
                    &dataset,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::Create {
                name,
                description,
                metadata,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let parsed_metadata: Option<serde_json::Value> = metadata
                    .as_ref()
                    .map(|m| serde_json::from_str(m))
                    .transpose()?;

                let client = LangfuseClient::new(&config)?;
                let dataset = client
                    .create_dataset(name, description.as_deref(), parsed_metadata.as_ref())
                    .await?;

                format_and_output(
                    &dataset,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::Items {
                dataset,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let items = client
                    .list_dataset_items(dataset.as_deref(), *limit, *page)
                    .await?;

                format_and_output(
                    &items,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::ItemGet {
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let item = client.get_dataset_item(id).await?;

                format_and_output(
                    &item,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::ItemCreate {
                dataset,
                input,
                expected_output,
                metadata,
                source_trace_id,
                source_observation_id,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let parsed_input: serde_json::Value = serde_json::from_str(input)?;
                let parsed_expected: Option<serde_json::Value> = expected_output
                    .as_ref()
                    .map(|e| serde_json::from_str(e))
                    .transpose()?;
                let parsed_metadata: Option<serde_json::Value> = metadata
                    .as_ref()
                    .map(|m| serde_json::from_str(m))
                    .transpose()?;

                let client = LangfuseClient::new(&config)?;
                let item = client
                    .create_dataset_item(
                        dataset,
                        &parsed_input,
                        parsed_expected.as_ref(),
                        parsed_metadata.as_ref(),
                        source_trace_id.as_deref(),
                        source_observation_id.as_deref(),
                    )
                    .await?;

                format_and_output(
                    &item,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::Runs {
                dataset,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let runs = client.list_dataset_runs(dataset, *limit, *page).await?;

                format_and_output(
                    &runs,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            DatasetsCommands::RunGet {
                dataset,
                run,
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
                    eprintln!(
                        "Error: Missing credentials. Run 'lf config setup' or set environment variables."
                    );
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;
                let run_data = client.get_dataset_run(dataset, run).await?;

                format_and_output(
                    &run_data,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }
        }
    }
}
