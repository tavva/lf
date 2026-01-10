use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub struct CsvFormatter;

impl CsvFormatter {
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

        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write header row
        wtr.write_record(&headers_vec)?;

        // Write data rows
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
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        let data = wtr.into_inner()?;
        Ok(String::from_utf8(data)?)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========== Basic Formatting Tests ==========

    #[test]
    fn test_format_empty_array() {
        let data: Vec<serde_json::Value> = vec![];
        let result = CsvFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_null() {
        let data: Option<String> = None;
        let result = CsvFormatter::format(&data).unwrap();
        assert_eq!(result, "No data to display");
    }

    #[test]
    fn test_format_single_object() {
        let data = json!({
            "id": "123",
            "name": "test"
        });
        let result = CsvFormatter::format(&data).unwrap();

        // Should have header row and data row
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("id"));
        assert!(lines[0].contains("name"));
        assert!(lines[1].contains("123"));
        assert!(lines[1].contains("test"));
    }

    #[test]
    fn test_format_array_of_objects() {
        let data = vec![
            json!({"id": "1", "status": "active"}),
            json!({"id": "2", "status": "inactive"}),
        ];
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 data rows

        // Header should contain both columns
        assert!(lines[0].contains("id"));
        assert!(lines[0].contains("status"));
    }

    #[test]
    fn test_format_primitive_value() {
        let data = "simple string";
        let result = CsvFormatter::format(&data).unwrap();
        assert!(result.contains("simple string"));
    }

    // ========== Value Formatting Tests ==========

    #[test]
    fn test_format_value_none() {
        let result = CsvFormatter::format_value(None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_null() {
        let result = CsvFormatter::format_value(Some(&Value::Null));
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_value_string() {
        let value = json!("hello");
        let result = CsvFormatter::format_value(Some(&value));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_value_number() {
        let value = json!(123);
        let result = CsvFormatter::format_value(Some(&value));
        assert_eq!(result, "123");

        let float_value = json!(45.67);
        let result = CsvFormatter::format_value(Some(&float_value));
        assert_eq!(result, "45.67");
    }

    #[test]
    fn test_format_value_boolean() {
        let true_val = json!(true);
        assert_eq!(CsvFormatter::format_value(Some(&true_val)), "true");

        let false_val = json!(false);
        assert_eq!(CsvFormatter::format_value(Some(&false_val)), "false");
    }

    #[test]
    fn test_format_value_array() {
        let value = json!([1, 2, 3]);
        let result = CsvFormatter::format_value(Some(&value));
        assert_eq!(result, "[1,2,3]");
    }

    #[test]
    fn test_format_value_object() {
        let value = json!({"a": 1});
        let result = CsvFormatter::format_value(Some(&value));
        assert_eq!(result, "{\"a\":1}");
    }

    // ========== CSV-Specific Tests ==========

    #[test]
    fn test_csv_comma_escaping() {
        let data = json!({
            "message": "hello, world"
        });
        let result = CsvFormatter::format(&data).unwrap();

        // Value with comma should be quoted in CSV
        assert!(result.contains("\"hello, world\""));
    }

    #[test]
    fn test_csv_quote_escaping() {
        let data = json!({
            "message": "say \"hello\""
        });
        let result = CsvFormatter::format(&data).unwrap();

        // Double quotes should be escaped
        assert!(result.contains("\"\""));
    }

    #[test]
    fn test_csv_newline_in_value() {
        let data = json!({
            "message": "line1\nline2"
        });
        let result = CsvFormatter::format(&data).unwrap();

        // Newlines in values should be preserved but quoted
        assert!(result.contains("line1\nline2") || result.contains("\"line1"));
    }

    #[test]
    fn test_csv_header_order() {
        let data = vec![
            json!({"zebra": "z", "alpha": "a", "middle": "m"}),
        ];
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        let headers: Vec<&str> = lines[0].split(',').collect();

        // Headers should be sorted alphabetically (BTreeSet)
        assert_eq!(headers, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn test_csv_objects_with_different_keys() {
        let data = vec![
            json!({"id": "1", "name": "Alice"}),
            json!({"id": "2", "email": "bob@test.com"}),
        ];
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();

        // Header should have all keys
        assert!(lines[0].contains("email"));
        assert!(lines[0].contains("id"));
        assert!(lines[0].contains("name"));

        // Each row should have correct number of columns
        assert_eq!(lines[1].split(',').count(), 3);
        assert_eq!(lines[2].split(',').count(), 3);
    }

    #[test]
    fn test_csv_empty_values() {
        let data = json!({
            "id": "1",
            "name": null
        });
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        // Should have two columns, one with empty value
        assert_eq!(lines[1].matches(',').count(), 1);
    }

    #[test]
    fn test_csv_numeric_values() {
        let data = json!({
            "int": 42,
            "float": 3.14,
            "negative": -100
        });
        let result = CsvFormatter::format(&data).unwrap();

        assert!(result.contains("42"));
        assert!(result.contains("3.14"));
        assert!(result.contains("-100"));
    }

    #[test]
    fn test_csv_unicode() {
        let data = json!({
            "greeting": "你好",
            "emoji": "🎉"
        });
        let result = CsvFormatter::format(&data).unwrap();

        assert!(result.contains("你好"));
        assert!(result.contains("🎉"));
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_csv_single_column() {
        let data = vec![
            json!({"only_column": "value1"}),
            json!({"only_column": "value2"}),
        ];
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].contains(',')); // Single column, no comma
    }

    #[test]
    fn test_csv_empty_string_values() {
        let data = json!({
            "id": "",
            "name": ""
        });
        let result = CsvFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        // Data row should just have a comma between empty values
        assert!(lines[1].contains(","));
    }

    #[test]
    fn test_csv_array_with_non_objects() {
        let data = vec![json!("string1"), json!("string2")];
        let result = CsvFormatter::format(&data).unwrap();
        // Should handle gracefully without error
        assert!(result.len() > 0);
    }

    #[test]
    fn test_csv_nested_object_as_json() {
        let data = json!({
            "id": "1",
            "metadata": {"key": "value"}
        });
        let result = CsvFormatter::format(&data).unwrap();

        // Nested object should be serialized as JSON string
        assert!(result.contains("{\"key\":\"value\"}") || result.contains("{\"\"key\"\":\"\"value\"\"}"));
    }
}
