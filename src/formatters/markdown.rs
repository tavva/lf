use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        let value = serde_json::to_value(data)?;

        match &value {
            Value::Array(arr) if arr.is_empty() => Ok("No data to display".to_string()),
            Value::Null => Ok("No data to display".to_string()),
            Value::Array(arr) => Self::format_array(arr),
            Value::Object(_) => Self::format_array(&[value]),
            _ => Ok(value.to_string()),
        }
    }

    fn format_array(arr: &[Value]) -> Result<String> {
        if arr.is_empty() {
            return Ok("No data to display".to_string());
        }

        // Collect all unique keys across all objects
        let mut headers: BTreeSet<String> = BTreeSet::new();
        for item in arr {
            if let Value::Object(obj) = item {
                for key in obj.keys() {
                    headers.insert(key.clone());
                }
            }
        }

        let headers_vec: Vec<String> = headers.into_iter().collect();

        let mut output = String::new();

        // Header row
        output.push('|');
        for header in &headers_vec {
            output.push_str(&format!(" {header} |"));
        }
        output.push('\n');

        // Separator row
        output.push('|');
        for _ in &headers_vec {
            output.push_str(" --- |");
        }
        output.push('\n');

        // Data rows
        for item in arr {
            output.push('|');
            for key in &headers_vec {
                let value = if let Value::Object(obj) = item {
                    Self::format_value(obj.get(key))
                } else {
                    String::new()
                };
                output.push_str(&format!(" {} |", Self::escape_pipes(&value)));
            }
            output.push('\n');
        }

        Ok(output)
    }

    fn format_value(value: Option<&Value>) -> String {
        match value {
            None | Some(Value::Null) => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Array(_)) | Some(Value::Object(_)) => {
                serde_json::to_string(value.unwrap()).unwrap_or_default()
            }
        }
    }

    fn escape_pipes(s: &str) -> String {
        s.replace('|', "\\|")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========== Basic Formatting Tests ==========

    #[test]
    fn test_format_empty_array() {
        let data: Vec<serde_json::Value> = vec![];
        let result = MarkdownFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_null() {
        let data: Option<String> = None;
        let result = MarkdownFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_single_object() {
        let data = json!({
            "id": "123",
            "name": "test"
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Should be valid markdown table
        assert!(result.contains("| id |"));
        assert!(result.contains("| name |"));
        assert!(result.contains("| --- |"));
        assert!(result.contains("| 123 |"));
        assert!(result.contains("| test |"));
    }

    #[test]
    fn test_format_array_of_objects() {
        let data = vec![
            json!({"id": "1", "status": "active"}),
            json!({"id": "2", "status": "inactive"}),
        ];
        let result = MarkdownFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // header + separator + 2 data rows

        // Header row
        assert!(lines[0].starts_with("|"));
        assert!(lines[0].ends_with("|"));

        // Separator row
        assert!(lines[1].contains("---"));
    }

    #[test]
    fn test_format_primitive_value() {
        let data = "simple string";
        let result = MarkdownFormatter::format(&data).unwrap();
        assert!(result.contains("simple string"));
    }

    // ========== Value Formatting Tests ==========

    #[test]
    fn test_format_value_none() {
        let result = MarkdownFormatter::format_value(None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_null() {
        let result = MarkdownFormatter::format_value(Some(&Value::Null));
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_string() {
        let value = json!("hello");
        let result = MarkdownFormatter::format_value(Some(&value));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_value_number() {
        let value = json!(123);
        let result = MarkdownFormatter::format_value(Some(&value));
        assert_eq!(result, "123");

        let float_value = json!(45.67);
        let result = MarkdownFormatter::format_value(Some(&float_value));
        assert_eq!(result, "45.67");
    }

    #[test]
    fn test_format_value_boolean() {
        let true_val = json!(true);
        assert_eq!(MarkdownFormatter::format_value(Some(&true_val)), "true");

        let false_val = json!(false);
        assert_eq!(MarkdownFormatter::format_value(Some(&false_val)), "false");
    }

    #[test]
    fn test_format_value_array() {
        let value = json!([1, 2, 3]);
        let result = MarkdownFormatter::format_value(Some(&value));
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_format_value_object() {
        let value = json!({"a": 1});
        let result = MarkdownFormatter::format_value(Some(&value));
        assert_eq!(result, "{\"a\":1}");
    }

    // ========== Pipe Escaping Tests ==========

    #[test]
    fn test_escape_pipes_no_pipes() {
        assert_eq!(
            MarkdownFormatter::escape_pipes("hello world"),
            "hello world"
        );
    }

    #[test]
    fn test_escape_pipes_single_pipe() {
        assert_eq!(MarkdownFormatter::escape_pipes("a|b"), "a\\|b");
    }

    #[test]
    fn test_escape_pipes_multiple_pipes() {
        assert_eq!(MarkdownFormatter::escape_pipes("a|b|c|d"), "a\\|b\\|c\\|d");
    }

    #[test]
    fn test_escape_pipes_only_pipe() {
        assert_eq!(MarkdownFormatter::escape_pipes("|"), "\\|");
    }

    #[test]
    fn test_escape_pipes_empty() {
        assert_eq!(MarkdownFormatter::escape_pipes(""), "");
    }

    // ========== Markdown Table Structure Tests ==========

    #[test]
    fn test_markdown_table_structure() {
        let data = vec![json!({"col1": "a", "col2": "b"})];
        let result = MarkdownFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();

        // Header row starts and ends with |
        assert!(lines[0].starts_with("|"));
        assert!(lines[0].ends_with("|"));

        // Separator row has proper format
        assert!(lines[1].starts_with("|"));
        assert!(lines[1].ends_with("|"));
        assert!(lines[1].contains("---"));

        // Data row starts and ends with |
        assert!(lines[2].starts_with("|"));
        assert!(lines[2].ends_with("|"));
    }

    #[test]
    fn test_markdown_column_count() {
        let data = vec![json!({"a": "1", "b": "2", "c": "3"})];
        let result = MarkdownFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();

        // Count pipes in header (should be 4 for 3 columns: |a|b|c|)
        assert_eq!(lines[0].matches('|').count(), 4);
        assert_eq!(lines[1].matches('|').count(), 4);
        assert_eq!(lines[2].matches('|').count(), 4);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_markdown_objects_with_different_keys() {
        let data = vec![
            json!({"id": "1", "name": "Alice"}),
            json!({"id": "2", "email": "bob@test.com"}),
        ];
        let result = MarkdownFormatter::format(&data).unwrap();

        // Should have all headers
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("email"));
    }

    #[test]
    fn test_markdown_value_with_pipe() {
        let data = json!({
            "expression": "a | b"
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Pipe in value should be escaped
        assert!(result.contains("a \\| b"));
    }

    #[test]
    fn test_markdown_empty_values() {
        let data = json!({
            "id": "1",
            "name": null
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Should still have proper table structure
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_markdown_unicode() {
        let data = json!({
            "greeting": "你好",
            "emoji": "🎉"
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        assert!(result.contains("你好"));
        assert!(result.contains("🎉"));
    }

    #[test]
    fn test_markdown_header_order() {
        let data = vec![json!({"zebra": "z", "alpha": "a", "middle": "m"})];
        let result = MarkdownFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        let header = lines[0];

        // Headers should be in sorted order
        let alpha_pos = header.find("alpha").unwrap();
        let middle_pos = header.find("middle").unwrap();
        let zebra_pos = header.find("zebra").unwrap();

        assert!(alpha_pos < middle_pos);
        assert!(middle_pos < zebra_pos);
    }

    #[test]
    fn test_markdown_nested_object() {
        let data = json!({
            "id": "1",
            "metadata": {"key": "value"}
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Nested object should be rendered as JSON
        assert!(result.contains("id"));
        assert!(result.contains("metadata"));
    }

    #[test]
    fn test_markdown_single_column() {
        let data = vec![json!({"only": "one"}), json!({"only": "two"})];
        let result = MarkdownFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // header + sep + 2 data
    }

    #[test]
    fn test_markdown_special_characters() {
        let data = json!({
            "code": "`inline`",
            "bold": "**text**"
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Markdown characters in values should be preserved
        assert!(result.contains("`inline`"));
        assert!(result.contains("**text**"));
    }

    #[test]
    fn test_markdown_long_values() {
        let data = json!({
            "short": "a",
            "long": "This is a very long value that spans many characters"
        });
        let result = MarkdownFormatter::format(&data).unwrap();

        // Long values should be preserved (no truncation in markdown)
        assert!(result.contains("This is a very long value"));
    }
}
