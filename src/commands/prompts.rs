// ABOUTME: Command handlers for prompt management operations
// ABOUTME: Supports list, get, create-text, create-chat, label, and delete

use anyhow::Result;
use clap::Subcommand;
use std::io::{self, Read};

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output, output_result};
use crate::types::{ChatMessage, OutputFormat, PromptContent};

#[derive(Debug, Subcommand)]
pub enum PromptsCommands {
    /// List prompts with optional filters
    List {
        /// Filter by prompt name
        #[arg(short, long)]
        name: Option<String>,

        /// Filter by label
        #[arg(short, long)]
        label: Option<String>,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: u32,

        /// Page number
        #[arg(long, default_value = "1")]
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

    /// Get a specific prompt by name
    Get {
        /// Prompt name
        name: String,

        /// Specific version number
        #[arg(long)]
        version: Option<i32>,

        /// Fetch by label (default: production)
        #[arg(short, long)]
        label: Option<String>,

        /// Output raw content only (for piping)
        #[arg(long)]
        raw: bool,

        /// Output format (ignored if --raw)
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

    /// Create a text prompt
    CreateText {
        /// Prompt name
        #[arg(long)]
        name: String,

        /// Read content from file (reads stdin if omitted)
        #[arg(short, long)]
        file: Option<String>,

        /// Commit message for this version
        #[arg(short, long)]
        message: Option<String>,

        /// Labels to apply
        #[arg(short, long)]
        labels: Option<Vec<String>>,

        /// Tags to apply
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Model config as JSON string
        #[arg(long)]
        config: Option<String>,

        /// Output format
        #[arg(long, value_enum)]
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

    /// Create a chat prompt
    CreateChat {
        /// Prompt name
        #[arg(long)]
        name: String,

        /// Read JSON messages from file (reads stdin if omitted)
        #[arg(short, long)]
        file: Option<String>,

        /// Commit message for this version
        #[arg(short, long)]
        message: Option<String>,

        /// Labels to apply
        #[arg(short, long)]
        labels: Option<Vec<String>>,

        /// Tags to apply
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Model config as JSON string
        #[arg(long)]
        config: Option<String>,

        /// Output format
        #[arg(long, value_enum)]
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

    /// Set labels on a prompt version
    Label {
        /// Prompt name
        name: String,

        /// Version number
        version: i32,

        /// Labels to set
        #[arg(short, long, required = true)]
        labels: Vec<String>,

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

    /// Delete a prompt
    Delete {
        /// Prompt name
        name: String,

        /// Delete specific version only
        #[arg(long)]
        version: Option<i32>,

        /// Delete versions with this label only
        #[arg(short, long)]
        label: Option<String>,

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

fn read_content(file: Option<&str>) -> Result<String> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

impl PromptsCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PromptsCommands::List {
                name,
                label,
                tag,
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

                let prompts = client
                    .list_prompts(
                        name.as_deref(),
                        label.as_deref(),
                        tag.as_deref(),
                        *limit,
                        *page,
                    )
                    .await?;

                format_and_output(
                    &prompts,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Get {
                name,
                version,
                label,
                raw,
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

                let prompt = client.get_prompt(name, *version, label.as_deref()).await?;

                if *raw {
                    let content = match &prompt.prompt {
                        PromptContent::Text(s) => s.clone(),
                        PromptContent::Chat(msgs) => serde_json::to_string_pretty(msgs)?,
                    };
                    output_result(&content, output.as_deref(), *verbose)
                } else {
                    format_and_output(
                        &prompt,
                        format.unwrap_or(OutputFormat::Json),
                        output.as_deref(),
                        *verbose,
                    )
                }
            }

            PromptsCommands::CreateText {
                name,
                file,
                message,
                labels,
                tags,
                config: cfg,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let app_config = build_config(
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

                if !app_config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let content = read_content(file.as_deref())?;
                let parsed_config: Option<serde_json::Value> =
                    cfg.as_ref().map(|c| serde_json::from_str(c)).transpose()?;

                let client = LangfuseClient::new(&app_config)?;

                let prompt = client
                    .create_text_prompt(
                        name,
                        &content,
                        labels.as_deref(),
                        tags.as_deref(),
                        parsed_config.as_ref(),
                        message.as_deref(),
                    )
                    .await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::CreateChat {
                name,
                file,
                message,
                labels,
                tags,
                config: cfg,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let app_config = build_config(
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

                if !app_config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let content = read_content(file.as_deref())?;
                let messages: Vec<ChatMessage> = serde_json::from_str(&content)?;
                let parsed_config: Option<serde_json::Value> =
                    cfg.as_ref().map(|c| serde_json::from_str(c)).transpose()?;

                let client = LangfuseClient::new(&app_config)?;

                let prompt = client
                    .create_chat_prompt(
                        name,
                        &messages,
                        labels.as_deref(),
                        tags.as_deref(),
                        parsed_config.as_ref(),
                        message.as_deref(),
                    )
                    .await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Label {
                name,
                version,
                labels,
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

                let prompt = client.update_prompt_labels(name, *version, labels).await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Delete {
                name,
                version,
                label,
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
                    None,
                    None,
                    None,
                    None,
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                client
                    .delete_prompt(name, *version, label.as_deref())
                    .await?;

                if *verbose {
                    eprintln!("Prompt '{}' deleted successfully", name);
                }

                Ok(())
            }
        }
    }
}
