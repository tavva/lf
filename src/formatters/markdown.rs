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
            output.push_str(&format!(" {} |", header));
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
