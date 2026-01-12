mod csv_formatter;
mod json;
mod markdown;
mod table;

pub use csv_formatter::CsvFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use table::TableFormatter;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_output_table() {
        let data = json!({"id": "1", "name": "test"});
        let result = format_output(&data, OutputFormat::Table).unwrap();

        // Table format should have structured output
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("1"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_format_output_json() {
        let data = json!({"id": "1", "name": "test"});
        let result = format_output(&data, OutputFormat::Json).unwrap();

        // JSON format should be valid JSON
        assert!(result.contains("\"id\": \"1\""));
        assert!(result.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_format_output_csv() {
        let data = json!({"id": "1", "name": "test"});
        let result = format_output(&data, OutputFormat::Csv).unwrap();

        // CSV format should have comma-separated values
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("1"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_format_output_markdown() {
        let data = json!({"id": "1", "name": "test"});
        let result = format_output(&data, OutputFormat::Markdown).unwrap();

        // Markdown format should have table structure
        assert!(result.contains("|"));
        assert!(result.contains("---"));
        assert!(result.contains("id"));
        assert!(result.contains("name"));
    }

    #[test]
    fn test_format_output_empty_data() {
        let data: Vec<serde_json::Value> = vec![];

        let table = format_output(&data, OutputFormat::Table).unwrap();
        let csv = format_output(&data, OutputFormat::Csv).unwrap();
        let markdown = format_output(&data, OutputFormat::Markdown).unwrap();
        let json = format_output(&data, OutputFormat::Json).unwrap();

        assert_eq!(table, "No data to display");
        assert_eq!(csv, "No data to display");
        assert_eq!(markdown, "No data to display");
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_format_output_array() {
        let data = vec![json!({"id": "1"}), json!({"id": "2"})];

        let table = format_output(&data, OutputFormat::Table).unwrap();
        let csv = format_output(&data, OutputFormat::Csv).unwrap();
        let markdown = format_output(&data, OutputFormat::Markdown).unwrap();
        let json = format_output(&data, OutputFormat::Json).unwrap();

        // All formats should include both records
        assert!(table.contains("1") && table.contains("2"));
        assert!(csv.contains("1") && csv.contains("2"));
        assert!(markdown.contains("1") && markdown.contains("2"));
        assert!(json.contains("1") && json.contains("2"));
    }

    #[test]
    fn test_format_output_complex_data() {
        let data = json!({
            "id": "trace-123",
            "name": "test-trace",
            "metadata": {"key": "value"},
            "tags": ["tag1", "tag2"],
            "count": 42,
            "active": true
        });

        // All formats should handle complex data without error
        assert!(format_output(&data, OutputFormat::Table).is_ok());
        assert!(format_output(&data, OutputFormat::Json).is_ok());
        assert!(format_output(&data, OutputFormat::Csv).is_ok());
        assert!(format_output(&data, OutputFormat::Markdown).is_ok());
    }
}
