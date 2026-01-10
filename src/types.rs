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
    #[serde(default)]
    pub observations: Vec<Observation>,
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
