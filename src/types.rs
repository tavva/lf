use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output format options
#[derive(Debug, Clone, Copy, Default, ValueEnum, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
    Markdown,
}

/// Metrics view options
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricsView {
    Traces,
    Observations,
}

/// Metrics measure options
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Measure {
    Count,
    Latency,
    InputTokens,
    OutputTokens,
    TotalTokens,
    InputCost,
    OutputCost,
    TotalCost,
}

impl Measure {
    pub fn to_api_string(&self) -> &str {
        match self {
            Measure::Count => "count",
            Measure::Latency => "latency",
            Measure::InputTokens => "inputTokens",
            Measure::OutputTokens => "outputTokens",
            Measure::TotalTokens => "totalTokens",
            Measure::InputCost => "inputCost",
            Measure::OutputCost => "outputCost",
            Measure::TotalCost => "totalCost",
        }
    }
}

/// Metrics aggregation options
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aggregation {
    Count,
    Sum,
    Avg,
    P50,
    P95,
    P99,
    Histogram,
}

impl Aggregation {
    pub fn to_api_string(&self) -> &str {
        match self {
            Aggregation::Count => "count",
            Aggregation::Sum => "sum",
            Aggregation::Avg => "avg",
            Aggregation::P50 => "p50",
            Aggregation::P95 => "p95",
            Aggregation::P99 => "p99",
            Aggregation::Histogram => "histogram",
        }
    }
}

/// Time granularity options
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeGranularity {
    Auto,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl TimeGranularity {
    pub fn to_api_string(&self) -> &str {
        match self {
            TimeGranularity::Auto => "auto",
            TimeGranularity::Minute => "minute",
            TimeGranularity::Hour => "hour",
            TimeGranularity::Day => "day",
            TimeGranularity::Week => "week",
            TimeGranularity::Month => "month",
        }
    }
}

/// Observation type options
#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObservationType {
    Generation,
    Span,
    Event,
}

impl ObservationType {
    pub fn to_api_string(&self) -> &str {
        match self {
            ObservationType::Generation => "GENERATION",
            ObservationType::Span => "SPAN",
            ObservationType::Event => "EVENT",
        }
    }
}

/// A trace from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub id: String,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub release: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub timestamp: Option<String>,
    /// Observations - can be IDs (strings) from list endpoint or full objects from get endpoint
    #[serde(default)]
    pub observations: Vec<serde_json::Value>,
}

/// A session from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub created_at: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub traces: Vec<Trace>,
}

/// An observation from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub id: String,
    pub trace_id: Option<String>,
    pub r#type: Option<String>,
    pub name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub model: Option<String>,
    pub model_parameters: Option<serde_json::Value>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub usage: Option<Usage>,
    pub level: Option<String>,
    pub status_message: Option<String>,
    pub parent_observation_id: Option<String>,
    pub completion_start_time: Option<String>,
}

/// Usage information for an observation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input: Option<i64>,
    pub output: Option<i64>,
    pub total: Option<i64>,
    pub unit: Option<String>,
    pub input_cost: Option<f64>,
    pub output_cost: Option<f64>,
    pub total_cost: Option<f64>,
}

/// A score from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub id: String,
    pub trace_id: Option<String>,
    pub observation_id: Option<String>,
    pub name: Option<String>,
    pub value: Option<serde_json::Value>,
    pub source: Option<String>,
    pub comment: Option<String>,
    pub timestamp: Option<String>,
    pub data_type: Option<String>,
    pub string_value: Option<String>,
}

/// Metrics query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResult {
    #[serde(default)]
    pub data: Vec<HashMap<String, serde_json::Value>>,
}

/// API response wrapper for traces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracesResponse {
    pub data: Vec<Trace>,
    pub meta: Option<PaginationMeta>,
}

/// API response wrapper for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub data: Vec<Session>,
    pub meta: Option<PaginationMeta>,
}

/// API response wrapper for observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationsResponse {
    pub data: Vec<Observation>,
    pub meta: Option<PaginationMeta>,
}

/// API response wrapper for scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoresResponse {
    pub data: Vec<Score>,
    pub meta: Option<PaginationMeta>,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub total_items: Option<i32>,
    pub total_pages: Option<i32>,
}

/// A chat message for chat prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Prompt content - either text or chat messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptContent {
    Text(String),
    Chat(Vec<ChatMessage>),
}

/// A prompt from Langfuse
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub name: String,
    pub version: i32,
    #[serde(rename = "type")]
    pub prompt_type: String,
    pub prompt: PromptContent,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub config: Option<serde_json::Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Prompt metadata from list endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptMeta {
    pub name: String,
    #[serde(default)]
    pub versions: Vec<i32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub last_updated_at: Option<String>,
}

/// API response wrapper for prompts list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsResponse {
    pub data: Vec<PromptMeta>,
    pub meta: Option<PaginationMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========== OutputFormat Tests ==========

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert_eq!(format, OutputFormat::Table);
    }

    #[test]
    fn test_output_format_serialize() {
        assert_eq!(
            serde_json::to_string(&OutputFormat::Table).unwrap(),
            "\"table\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::Csv).unwrap(),
            "\"csv\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::Markdown).unwrap(),
            "\"markdown\""
        );
    }

    #[test]
    fn test_output_format_deserialize() {
        assert_eq!(
            serde_json::from_str::<OutputFormat>("\"table\"").unwrap(),
            OutputFormat::Table
        );
        assert_eq!(
            serde_json::from_str::<OutputFormat>("\"json\"").unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            serde_json::from_str::<OutputFormat>("\"csv\"").unwrap(),
            OutputFormat::Csv
        );
        assert_eq!(
            serde_json::from_str::<OutputFormat>("\"markdown\"").unwrap(),
            OutputFormat::Markdown
        );
    }

    // ========== Measure Tests ==========

    #[test]
    fn test_measure_to_api_string() {
        assert_eq!(Measure::Count.to_api_string(), "count");
        assert_eq!(Measure::Latency.to_api_string(), "latency");
        assert_eq!(Measure::InputTokens.to_api_string(), "inputTokens");
        assert_eq!(Measure::OutputTokens.to_api_string(), "outputTokens");
        assert_eq!(Measure::TotalTokens.to_api_string(), "totalTokens");
        assert_eq!(Measure::InputCost.to_api_string(), "inputCost");
        assert_eq!(Measure::OutputCost.to_api_string(), "outputCost");
        assert_eq!(Measure::TotalCost.to_api_string(), "totalCost");
    }

    #[test]
    fn test_measure_serialize() {
        assert_eq!(serde_json::to_string(&Measure::Count).unwrap(), "\"count\"");
        assert_eq!(
            serde_json::to_string(&Measure::InputTokens).unwrap(),
            "\"inputtokens\""
        );
        assert_eq!(
            serde_json::to_string(&Measure::TotalCost).unwrap(),
            "\"totalcost\""
        );
    }

    // ========== Aggregation Tests ==========

    #[test]
    fn test_aggregation_to_api_string() {
        assert_eq!(Aggregation::Count.to_api_string(), "count");
        assert_eq!(Aggregation::Sum.to_api_string(), "sum");
        assert_eq!(Aggregation::Avg.to_api_string(), "avg");
        assert_eq!(Aggregation::P50.to_api_string(), "p50");
        assert_eq!(Aggregation::P95.to_api_string(), "p95");
        assert_eq!(Aggregation::P99.to_api_string(), "p99");
        assert_eq!(Aggregation::Histogram.to_api_string(), "histogram");
    }

    #[test]
    fn test_aggregation_serialize() {
        assert_eq!(
            serde_json::to_string(&Aggregation::Count).unwrap(),
            "\"count\""
        );
        assert_eq!(serde_json::to_string(&Aggregation::P95).unwrap(), "\"p95\"");
    }

    // ========== TimeGranularity Tests ==========

    #[test]
    fn test_time_granularity_to_api_string() {
        assert_eq!(TimeGranularity::Auto.to_api_string(), "auto");
        assert_eq!(TimeGranularity::Minute.to_api_string(), "minute");
        assert_eq!(TimeGranularity::Hour.to_api_string(), "hour");
        assert_eq!(TimeGranularity::Day.to_api_string(), "day");
        assert_eq!(TimeGranularity::Week.to_api_string(), "week");
        assert_eq!(TimeGranularity::Month.to_api_string(), "month");
    }

    #[test]
    fn test_time_granularity_serialize() {
        assert_eq!(
            serde_json::to_string(&TimeGranularity::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&TimeGranularity::Month).unwrap(),
            "\"month\""
        );
    }

    // ========== ObservationType Tests ==========

    #[test]
    fn test_observation_type_to_api_string() {
        assert_eq!(ObservationType::Generation.to_api_string(), "GENERATION");
        assert_eq!(ObservationType::Span.to_api_string(), "SPAN");
        assert_eq!(ObservationType::Event.to_api_string(), "EVENT");
    }

    #[test]
    fn test_observation_type_serialize() {
        assert_eq!(
            serde_json::to_string(&ObservationType::Generation).unwrap(),
            "\"GENERATION\""
        );
        assert_eq!(
            serde_json::to_string(&ObservationType::Span).unwrap(),
            "\"SPAN\""
        );
        assert_eq!(
            serde_json::to_string(&ObservationType::Event).unwrap(),
            "\"EVENT\""
        );
    }

    // ========== Trace Tests ==========

    #[test]
    fn test_trace_deserialize() {
        let json = json!({
            "id": "trace-123",
            "name": "my-trace",
            "userId": "user-456",
            "sessionId": "session-789",
            "release": "v1.0.0",
            "version": "1",
            "metadata": {"key": "value"},
            "tags": ["tag1", "tag2"],
            "input": {"prompt": "Hello"},
            "output": {"response": "World"},
            "timestamp": "2024-01-15T10:30:00Z"
        });

        let trace: Trace = serde_json::from_value(json).unwrap();

        assert_eq!(trace.id, "trace-123");
        assert_eq!(trace.name, Some("my-trace".to_string()));
        assert_eq!(trace.user_id, Some("user-456".to_string()));
        assert_eq!(trace.session_id, Some("session-789".to_string()));
        assert_eq!(trace.release, Some("v1.0.0".to_string()));
        assert_eq!(
            trace.tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
        assert!(trace.observations.is_empty());
    }

    #[test]
    fn test_trace_deserialize_minimal() {
        let json = json!({
            "id": "trace-min"
        });

        let trace: Trace = serde_json::from_value(json).unwrap();

        assert_eq!(trace.id, "trace-min");
        assert!(trace.name.is_none());
        assert!(trace.user_id.is_none());
        assert!(trace.observations.is_empty());
    }

    #[test]
    fn test_trace_serialize() {
        let trace = Trace {
            id: "trace-123".to_string(),
            name: Some("test".to_string()),
            user_id: None,
            session_id: None,
            release: None,
            version: None,
            metadata: None,
            tags: Some(vec!["tag1".to_string()]),
            input: None,
            output: None,
            timestamp: None,
            observations: vec![],
        };

        let json = serde_json::to_value(&trace).unwrap();

        assert_eq!(json["id"], "trace-123");
        assert_eq!(json["name"], "test");
        assert!(json["tags"].as_array().unwrap().contains(&json!("tag1")));
    }

    // ========== Session Tests ==========

    #[test]
    fn test_session_deserialize() {
        let json = json!({
            "id": "session-123",
            "createdAt": "2024-01-15T10:30:00Z",
            "projectId": "project-456"
        });

        let session: Session = serde_json::from_value(json).unwrap();

        assert_eq!(session.id, "session-123");
        assert_eq!(session.created_at, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(session.project_id, Some("project-456".to_string()));
        assert!(session.traces.is_empty());
    }

    #[test]
    fn test_session_with_traces() {
        let json = json!({
            "id": "session-123",
            "traces": [
                {"id": "trace-1"},
                {"id": "trace-2"}
            ]
        });

        let session: Session = serde_json::from_value(json).unwrap();

        assert_eq!(session.traces.len(), 2);
        assert_eq!(session.traces[0].id, "trace-1");
        assert_eq!(session.traces[1].id, "trace-2");
    }

    // ========== Observation Tests ==========

    #[test]
    fn test_observation_deserialize() {
        let json = json!({
            "id": "obs-123",
            "traceId": "trace-456",
            "type": "GENERATION",
            "name": "my-observation",
            "startTime": "2024-01-15T10:30:00Z",
            "endTime": "2024-01-15T10:30:01Z",
            "model": "gpt-4",
            "level": "DEFAULT"
        });

        let obs: Observation = serde_json::from_value(json).unwrap();

        assert_eq!(obs.id, "obs-123");
        assert_eq!(obs.trace_id, Some("trace-456".to_string()));
        assert_eq!(obs.r#type, Some("GENERATION".to_string()));
        assert_eq!(obs.name, Some("my-observation".to_string()));
        assert_eq!(obs.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_observation_with_usage() {
        let json = json!({
            "id": "obs-123",
            "usage": {
                "input": 100,
                "output": 50,
                "total": 150,
                "unit": "TOKENS",
                "inputCost": 0.001,
                "outputCost": 0.002,
                "totalCost": 0.003
            }
        });

        let obs: Observation = serde_json::from_value(json).unwrap();

        let usage = obs.usage.unwrap();
        assert_eq!(usage.input, Some(100));
        assert_eq!(usage.output, Some(50));
        assert_eq!(usage.total, Some(150));
        assert_eq!(usage.unit, Some("TOKENS".to_string()));
        assert_eq!(usage.input_cost, Some(0.001));
        assert_eq!(usage.output_cost, Some(0.002));
        assert_eq!(usage.total_cost, Some(0.003));
    }

    // ========== Score Tests ==========

    #[test]
    fn test_score_deserialize() {
        let json = json!({
            "id": "score-123",
            "traceId": "trace-456",
            "observationId": "obs-789",
            "name": "accuracy",
            "value": 0.95,
            "source": "API",
            "comment": "Test score",
            "timestamp": "2024-01-15T10:30:00Z",
            "dataType": "NUMERIC"
        });

        let score: Score = serde_json::from_value(json).unwrap();

        assert_eq!(score.id, "score-123");
        assert_eq!(score.trace_id, Some("trace-456".to_string()));
        assert_eq!(score.observation_id, Some("obs-789".to_string()));
        assert_eq!(score.name, Some("accuracy".to_string()));
        assert_eq!(score.source, Some("API".to_string()));
        assert_eq!(score.data_type, Some("NUMERIC".to_string()));
    }

    #[test]
    fn test_score_with_string_value() {
        let json = json!({
            "id": "score-123",
            "stringValue": "good"
        });

        let score: Score = serde_json::from_value(json).unwrap();

        assert_eq!(score.string_value, Some("good".to_string()));
    }

    // ========== Usage Tests ==========

    #[test]
    fn test_usage_deserialize() {
        let json = json!({
            "input": 500,
            "output": 200,
            "total": 700,
            "unit": "TOKENS",
            "inputCost": 0.01,
            "outputCost": 0.02,
            "totalCost": 0.03
        });

        let usage: Usage = serde_json::from_value(json).unwrap();

        assert_eq!(usage.input, Some(500));
        assert_eq!(usage.output, Some(200));
        assert_eq!(usage.total, Some(700));
        assert_eq!(usage.unit, Some("TOKENS".to_string()));
    }

    #[test]
    fn test_usage_partial() {
        let json = json!({
            "input": 100
        });

        let usage: Usage = serde_json::from_value(json).unwrap();

        assert_eq!(usage.input, Some(100));
        assert!(usage.output.is_none());
        assert!(usage.total.is_none());
    }

    // ========== MetricsResult Tests ==========

    #[test]
    fn test_metrics_result_deserialize() {
        let json = json!({
            "data": [
                {"name": "metric1", "value": 100},
                {"name": "metric2", "value": 200}
            ]
        });

        let result: MetricsResult = serde_json::from_value(json).unwrap();

        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].get("name").unwrap(), &json!("metric1"));
        assert_eq!(result.data[1].get("value").unwrap(), &json!(200));
    }

    #[test]
    fn test_metrics_result_empty() {
        let json = json!({});

        let result: MetricsResult = serde_json::from_value(json).unwrap();

        assert!(result.data.is_empty());
    }

    // ========== Response Wrapper Tests ==========

    #[test]
    fn test_traces_response_deserialize() {
        let json = json!({
            "data": [
                {"id": "trace-1"},
                {"id": "trace-2"}
            ],
            "meta": {
                "page": 1,
                "limit": 50,
                "totalItems": 100,
                "totalPages": 2
            }
        });

        let response: TracesResponse = serde_json::from_value(json).unwrap();

        assert_eq!(response.data.len(), 2);
        let meta = response.meta.unwrap();
        assert_eq!(meta.page, Some(1));
        assert_eq!(meta.limit, Some(50));
        assert_eq!(meta.total_items, Some(100));
        assert_eq!(meta.total_pages, Some(2));
    }

    #[test]
    fn test_sessions_response_deserialize() {
        let json = json!({
            "data": [
                {"id": "session-1"},
                {"id": "session-2"}
            ]
        });

        let response: SessionsResponse = serde_json::from_value(json).unwrap();

        assert_eq!(response.data.len(), 2);
        assert!(response.meta.is_none());
    }

    #[test]
    fn test_observations_response_deserialize() {
        let json = json!({
            "data": [
                {"id": "obs-1", "type": "GENERATION"},
                {"id": "obs-2", "type": "SPAN"}
            ],
            "meta": {
                "page": 2,
                "totalPages": 5
            }
        });

        let response: ObservationsResponse = serde_json::from_value(json).unwrap();

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].r#type, Some("GENERATION".to_string()));
    }

    #[test]
    fn test_scores_response_deserialize() {
        let json = json!({
            "data": [
                {"id": "score-1", "name": "accuracy"},
                {"id": "score-2", "name": "relevance"}
            ]
        });

        let response: ScoresResponse = serde_json::from_value(json).unwrap();

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, Some("accuracy".to_string()));
    }

    // ========== PaginationMeta Tests ==========

    #[test]
    fn test_pagination_meta_deserialize() {
        let json = json!({
            "page": 3,
            "limit": 100,
            "totalItems": 500,
            "totalPages": 5
        });

        let meta: PaginationMeta = serde_json::from_value(json).unwrap();

        assert_eq!(meta.page, Some(3));
        assert_eq!(meta.limit, Some(100));
        assert_eq!(meta.total_items, Some(500));
        assert_eq!(meta.total_pages, Some(5));
    }

    #[test]
    fn test_pagination_meta_partial() {
        let json = json!({
            "page": 1
        });

        let meta: PaginationMeta = serde_json::from_value(json).unwrap();

        assert_eq!(meta.page, Some(1));
        assert!(meta.limit.is_none());
        assert!(meta.total_items.is_none());
        assert!(meta.total_pages.is_none());
    }

    // ========== Prompt Tests ==========

    #[test]
    fn test_chat_message_deserialize() {
        let json = json!({
            "role": "system",
            "content": "You are a helpful assistant."
        });

        let msg: ChatMessage = serde_json::from_value(json).unwrap();

        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "You are a helpful assistant.");
    }

    #[test]
    fn test_prompt_text_deserialize() {
        let json = json!({
            "name": "welcome",
            "version": 3,
            "type": "text",
            "prompt": "Hello {{name}}!",
            "labels": ["production"],
            "tags": ["greeting"],
            "config": {"temperature": 0.7},
            "createdAt": "2024-01-15T10:00:00Z",
            "updatedAt": "2024-01-15T10:00:00Z"
        });

        let prompt: Prompt = serde_json::from_value(json).unwrap();

        assert_eq!(prompt.name, "welcome");
        assert_eq!(prompt.version, 3);
        assert_eq!(prompt.prompt_type, "text");
        assert_eq!(prompt.labels, vec!["production"]);
        match prompt.prompt {
            PromptContent::Text(s) => assert_eq!(s, "Hello {{name}}!"),
            _ => panic!("Expected text prompt"),
        }
    }

    #[test]
    fn test_prompt_chat_deserialize() {
        let json = json!({
            "name": "assistant",
            "version": 1,
            "type": "chat",
            "prompt": [
                {"role": "system", "content": "You are helpful."},
                {"role": "user", "content": "{{question}}"}
            ],
            "labels": [],
            "tags": [],
            "createdAt": "2024-01-15T10:00:00Z",
            "updatedAt": "2024-01-15T10:00:00Z"
        });

        let prompt: Prompt = serde_json::from_value(json).unwrap();

        assert_eq!(prompt.prompt_type, "chat");
        match prompt.prompt {
            PromptContent::Chat(msgs) => {
                assert_eq!(msgs.len(), 2);
                assert_eq!(msgs[0].role, "system");
            }
            _ => panic!("Expected chat prompt"),
        }
    }

    #[test]
    fn test_prompt_meta_deserialize() {
        let json = json!({
            "name": "welcome",
            "versions": [1, 2, 3],
            "labels": ["production", "staging"],
            "tags": ["greeting"],
            "lastUpdatedAt": "2024-01-15T10:00:00Z"
        });

        let meta: PromptMeta = serde_json::from_value(json).unwrap();

        assert_eq!(meta.name, "welcome");
        assert_eq!(meta.versions, vec![1, 2, 3]);
        assert_eq!(meta.labels, vec!["production", "staging"]);
    }

    #[test]
    fn test_prompts_response_deserialize() {
        let json = json!({
            "data": [
                {"name": "p1", "versions": [1], "labels": [], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"},
                {"name": "p2", "versions": [1, 2], "labels": ["prod"], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"}
            ],
            "meta": {
                "page": 1,
                "limit": 50,
                "totalItems": 2,
                "totalPages": 1
            }
        });

        let response: PromptsResponse = serde_json::from_value(json).unwrap();

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "p1");
    }
}
