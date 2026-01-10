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
