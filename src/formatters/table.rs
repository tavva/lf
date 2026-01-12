use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use tabled::{builder::Builder, settings::Style};

pub struct TableFormatter;

impl TableFormatter {
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

        // No object keys found - can't display as table
        if headers.is_empty() {
            return Ok("No data to display".to_string());
        }

        let headers_vec: Vec<String> = headers.into_iter().collect();

        let mut builder = Builder::default();

        // Add header row
        builder.push_record(headers_vec.iter().map(|s| s.as_str()));

        // Add data rows
        for item in arr {
            let row: Vec<String> = headers_vec
                .iter()
                .map(|key| {
                    if let Value::Object(obj) = item {
                        Self::format_value(obj.get(key))
                    } else {
                        String::new()
                    }
                })
                .collect();
            builder.push_record(row);
        }

        let mut table = builder.build();
        table.with(Style::rounded());

        Ok(table.to_string())
    }

    fn format_value(value: Option<&Value>) -> String {
        match value {
            None | Some(Value::Null) => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Array(arr)) => {
                // Truncate long arrays
                let s = serde_json::to_string(arr).unwrap_or_default();
                Self::truncate_string(&s, 50)
            }
            Some(Value::Object(obj)) => {
                // Truncate long objects
                let s = serde_json::to_string(obj).unwrap_or_default();
                Self::truncate_string(&s, 50)
            }
        }
    }

    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
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
        let result = TableFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_null() {
        let data: Option<String> = None;
        let result = TableFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_single_object() {
        let data = json!({
            "id": "123",
            "name": "test"
        });
        let result = TableFormatter::format(&data).unwrap();

        // Should contain table formatting and data
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("123"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_format_array_of_objects() {
        let data = vec![
            json!({"id": "1", "status": "active"}),
            json!({"id": "2", "status": "inactive"}),
        ];
        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("id"));
        assert!(result.contains("status"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("active"));
        assert!(result.contains("inactive"));
    }

    #[test]
    fn test_format_primitive_value() {
        let data = "simple string";
        let result = TableFormatter::format(&data).unwrap();
        assert!(result.contains("simple string"));
    }

    #[test]
    fn test_format_number() {
        let data = 42;
        let result = TableFormatter::format(&data).unwrap();
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_boolean() {
        let data = true;
        let result = TableFormatter::format(&data).unwrap();
        assert!(result.contains("true"));
    }

    // ========== Value Formatting Tests ==========

    #[test]
    fn test_format_value_none() {
        let result = TableFormatter::format_value(None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_null() {
        let result = TableFormatter::format_value(Some(&Value::Null));
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_string() {
        let value = json!("hello");
        let result = TableFormatter::format_value(Some(&value));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_value_number() {
        let value = json!(123);
        let result = TableFormatter::format_value(Some(&value));
        assert_eq!(result, "123");

        let float_value = json!(45.67);
        let result = TableFormatter::format_value(Some(&float_value));
        assert_eq!(result, "45.67");
    }

    #[test]
    fn test_format_value_boolean() {
        let true_val = json!(true);
        assert_eq!(TableFormatter::format_value(Some(&true_val)), "true");

        let false_val = json!(false);
        assert_eq!(TableFormatter::format_value(Some(&false_val)), "false");
    }

    #[test]
    fn test_format_value_array_short() {
        let value = json!([1, 2, 3]);
        let result = TableFormatter::format_value(Some(&value));
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_format_value_array_long() {
        let value = json!([
            "this",
            "is",
            "a",
            "very",
            "long",
            "array",
            "that",
            "should",
            "be",
            "truncated"
        ]);
        let result = TableFormatter::format_value(Some(&value));
        assert!(result.ends_with("..."));
        assert!(result.len() <= 53); // 50 + "..."
    }

    #[test]
    fn test_format_value_object_short() {
        let value = json!({"a": 1});
        let result = TableFormatter::format_value(Some(&value));
        assert_eq!(result, "{\"a\":1}");
    }

    #[test]
    fn test_format_value_object_long() {
        let value = json!({
            "long_key_name": "this is a very long value that should be truncated"
        });
        let result = TableFormatter::format_value(Some(&value));
        assert!(result.ends_with("..."));
        assert!(result.len() <= 53); // 50 + "..."
    }

    // ========== Truncation Tests ==========

    #[test]
    fn test_truncate_string_short() {
        let result = TableFormatter::truncate_string("short", 50);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_string_exact() {
        let s = "x".repeat(50);
        let result = TableFormatter::truncate_string(&s, 50);
        assert_eq!(result, s);
    }

    #[test]
    fn test_truncate_string_long() {
        let s = "x".repeat(100);
        let result = TableFormatter::truncate_string(&s, 50);
        assert_eq!(result.len(), 53); // 50 + "..."
        assert!(result.ends_with("..."));
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_format_objects_with_different_keys() {
        let data = vec![
            json!({"id": "1", "name": "Alice"}),
            json!({"id": "2", "email": "bob@test.com"}),
        ];
        let result = TableFormatter::format(&data).unwrap();

        // Should contain all keys from both objects
        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("email"));
    }

    #[test]
    fn test_format_nested_object() {
        let data = json!({
            "id": "1",
            "metadata": {"key": "value"}
        });
        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("id"));
        assert!(result.contains("metadata"));
        assert!(result.contains("1"));
    }

    #[test]
    fn test_format_with_empty_strings() {
        let data = json!({
            "id": "",
            "name": ""
        });
        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("id"));
        assert!(result.contains("name"));
    }

    #[test]
    fn test_format_array_with_non_objects() {
        let data = vec![json!("string1"), json!("string2")];
        let result = TableFormatter::format(&data).unwrap();
        // Non-objects can't be displayed as a table
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_special_characters() {
        let data = json!({
            "message": "Hello\nWorld\tTab"
        });
        let result = TableFormatter::format(&data).unwrap();
        assert!(result.contains("message"));
    }
}
