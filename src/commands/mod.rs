pub mod config;
pub mod traces;
pub mod sessions;
pub mod observations;
pub mod scores;
pub mod metrics;

use anyhow::Result;
use std::fs;

use crate::config::Config;
use crate::formatters::format_output;
use crate::types::OutputFormat;

/// Output result to stdout or file
pub fn output_result(content: &str, output_path: Option<&str>, verbose: bool) -> Result<()> {
    if let Some(path) = output_path {
        fs::write(path, content)?;
        if verbose {
            eprintln!("Output written to: {}", path);
        }
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Format and output data
pub fn format_and_output<T: serde::Serialize>(
    data: &T,
    format: OutputFormat,
    output_path: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let formatted = format_output(data, format)?;
    output_result(&formatted, output_path, verbose)
}

/// Helper to build config from CLI args
pub fn build_config(
    profile: Option<&str>,
    public_key: Option<&str>,
    secret_key: Option<&str>,
    host: Option<&str>,
    format: Option<OutputFormat>,
    limit: Option<u32>,
    page: Option<u32>,
    output: Option<&str>,
    verbose: bool,
    no_color: bool,
) -> Result<Config> {
    Config::load(
        profile, public_key, secret_key, host, format, limit, page, output, verbose, no_color,
    )
}
