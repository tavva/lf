use anyhow::Result;
use clap::Subcommand;

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output};
use crate::types::{Aggregation, Measure, MetricsView, OutputFormat, TimeGranularity};

#[derive(Debug, Subcommand)]
pub enum MetricsCommands {
    /// Query metrics with aggregations
    Query {
        /// View to query (traces or observations)
        #[arg(long, value_enum)]
        view: MetricsView,

        /// Measure to aggregate
        #[arg(long, value_enum)]
        measure: Measure,

        /// Aggregation function
        #[arg(long, value_enum)]
        aggregation: Aggregation,

        /// Dimensions for grouping (can be specified multiple times)
        #[arg(short, long)]
        dimensions: Option<Vec<String>>,

        /// Filter from timestamp (ISO 8601 format)
        #[arg(long)]
        from: Option<String>,

        /// Filter to timestamp (ISO 8601 format)
        #[arg(long)]
        to: Option<String>,

        /// Time granularity for bucketing
        #[arg(long, value_enum)]
        granularity: Option<TimeGranularity>,

        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<u32>,

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

impl MetricsCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            MetricsCommands::Query {
                view,
                measure,
                aggregation,
                dimensions,
                from,
                to,
                granularity,
                limit,
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

                // Convert view to API string
                let view_str = match view {
                    MetricsView::Traces => "traces",
                    MetricsView::Observations => "observations",
                };

                let result = client
                    .query_metrics(
                        view_str,
                        measure.to_api_string(),
                        aggregation.to_api_string(),
                        dimensions.as_deref(),
                        from.as_deref(),
                        to.as_deref(),
                        granularity.as_ref().map(|g| g.to_api_string()),
                        *limit,
                    )
                    .await?;

                format_and_output(
                    &result.data,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }
        }
    }
}
