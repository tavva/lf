use anyhow::Result;
use serde::Serialize;

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        Ok(serde_json::to_string_pretty(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_simple_object() {
        let data = json!({
            "id": "123",
            "name": "test"
        });
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("\"id\": \"123\""));
        assert!(result.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_format_array() {
        let data = vec![json!({"id": "1"}), json!({"id": "2"})];
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("["));
        assert!(result.contains("]"));
        assert!(result.contains("\"id\": \"1\""));
        assert!(result.contains("\"id\": \"2\""));
    }

    #[test]
    fn test_format_empty_array() {
        let data: Vec<serde_json::Value> = vec![];
        let result = JsonFormatter::format(&data).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_format_null() {
        let data: Option<String> = None;
        let result = JsonFormatter::format(&data).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_format_string() {
        let data = "hello world";
        let result = JsonFormatter::format(&data).unwrap();
        assert_eq!(result, "\"hello world\"");
    }

    #[test]
    fn test_format_number() {
        let data = 42;
        let result = JsonFormatter::format(&data).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_boolean() {
        assert_eq!(JsonFormatter::format(&true).unwrap(), "true");
        assert_eq!(JsonFormatter::format(&false).unwrap(), "false");
    }

    #[test]
    fn test_format_nested_object() {
        let data = json!({
            "outer": {
                "inner": {
                    "value": 123
                }
            }
        });
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("outer"));
        assert!(result.contains("inner"));
        assert!(result.contains("123"));
    }

    #[test]
    fn test_format_pretty_print() {
        let data = json!({"a": 1, "b": 2});
        let result = JsonFormatter::format(&data).unwrap();

        // Pretty-printed JSON should have newlines and indentation
        assert!(result.contains("\n"));
        assert!(result.contains("  ")); // indentation
    }

    #[test]
    fn test_format_special_characters() {
        let data = json!({
            "message": "Hello\nWorld\t\"Quoted\""
        });
        let result = JsonFormatter::format(&data).unwrap();

        // Special characters should be properly escaped
        assert!(result.contains("\\n"));
        assert!(result.contains("\\t"));
        assert!(result.contains("\\\""));
    }

    #[test]
    fn test_format_unicode() {
        let data = json!({
            "greeting": "你好世界",
            "emoji": "🎉"
        });
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("你好世界"));
        assert!(result.contains("🎉"));
    }

    #[test]
    fn test_format_large_numbers() {
        let data = json!({
            "big_int": 9007199254740993_i64,
            "float": 3.141592653589793
        });
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("9007199254740993"));
        assert!(result.contains("3.141592653589793"));
    }

    #[derive(Serialize)]
    struct CustomStruct {
        name: String,
        count: i32,
        active: bool,
    }

    #[test]
    fn test_format_custom_struct() {
        let data = CustomStruct {
            name: "test".to_string(),
            count: 42,
            active: true,
        };
        let result = JsonFormatter::format(&data).unwrap();

        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"count\": 42"));
        assert!(result.contains("\"active\": true"));
    }
}
