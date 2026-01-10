use anyhow::Result;
use serde::Serialize;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}
