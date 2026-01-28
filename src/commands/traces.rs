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

        /// Strip large content fields (input, output) from observations
        #[arg(long)]
        summary: bool,

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
                summary,
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
                        .list_observations(Some(id), None, None, None, None, None, 100, 1)
                        .await?;
                    trace.observations = observations
                        .into_iter()
                        .map(|o| {
                            let value = serde_json::to_value(o).unwrap_or_default();
                            if *summary {
                                strip_observation_content(value)
                            } else {
                                value
                            }
                        })
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

/// Strips large content fields (input, output) from an observation JSON value.
fn strip_observation_content(mut obs: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = obs.as_object_mut() {
        obj.remove("input");
        obj.remove("output");
    }
    obs
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_strip_observation_content_removes_input_output() {
        let obs = json!({
            "id": "obs-123",
            "trace_id": "trace-456",
            "type": "GENERATION",
            "name": "chat-completion",
            "model": "gpt-4",
            "input": {"messages": [{"role": "user", "content": "Hello, how are you?"}]},
            "output": {"content": "I'm doing well, thank you for asking!"},
            "usage": {"input_tokens": 10, "output_tokens": 15}
        });

        let result = strip_observation_content(obs);

        assert!(
            result.get("input").is_none(),
            "input field should be removed"
        );
        assert!(
            result.get("output").is_none(),
            "output field should be removed"
        );
        assert_eq!(result.get("id").unwrap(), "obs-123");
        assert_eq!(result.get("trace_id").unwrap(), "trace-456");
        assert_eq!(result.get("type").unwrap(), "GENERATION");
        assert_eq!(result.get("name").unwrap(), "chat-completion");
        assert_eq!(result.get("model").unwrap(), "gpt-4");
        assert!(
            result.get("usage").is_some(),
            "usage field should be preserved"
        );
    }

    #[test]
    fn test_strip_observation_content_handles_missing_fields() {
        let obs = json!({
            "id": "obs-123",
            "type": "SPAN"
        });

        let result = strip_observation_content(obs);

        assert_eq!(result.get("id").unwrap(), "obs-123");
        assert_eq!(result.get("type").unwrap(), "SPAN");
    }
}
