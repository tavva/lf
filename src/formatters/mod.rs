mod table;
mod csv_formatter;
mod markdown;
mod json;

pub use table::TableFormatter;
pub use csv_formatter::CsvFormatter;
pub use markdown::MarkdownFormatter;
pub use json::JsonFormatter;

use anyhow::Result;
use serde::Serialize;

use crate::types::OutputFormat;

/// Format data according to the specified output format
pub fn format_output<T: Serialize>(data: &T, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Table => TableFormatter::format(data),
        OutputFormat::Json => JsonFormatter::format(data),
        OutputFormat::Csv => CsvFormatter::format(data),
        OutputFormat::Markdown => MarkdownFormatter::format(data),
    }
}
