use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use thiserror::Error;

use crate::config::Config;
use crate::types::*;

/// API errors
#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ApiError {
    #[error("Authentication failed. Check your public and secret keys.")]
    AuthenticationError,

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Rate limit exceeded. Please try again later.")]
    RateLimitError,

    #[error("Request timeout")]
    TimeoutError,

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// Langfuse API client
#[derive(Debug)]
pub struct LangfuseClient {
    client: Client,
    host: String,
    public_key: String,
    secret_key: String,
}

impl LangfuseClient {
    /// Create a new client from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let public_key = config
            .public_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Public key is required"))?;
        let secret_key = config
            .secret_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Secret key is required"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            host: config.host.clone(),
            public_key,
            secret_key,
        })
    }

    /// Make an authenticated GET request
    async fn get<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> {
        let url = format!("{}/api/public{}", self.host, path);

        let mut request = self
            .client
            .get(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key));

        if !params.is_empty() {
            request = request.query(params);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::TimeoutError
            } else {
                ApiError::NetworkError(e.to_string())
            }
        })?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let body = response
                    .json::<T>()
                    .await
                    .context("Failed to parse response")?;
                Ok(body)
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(ApiError::AuthenticationError.into())
            }
            StatusCode::NOT_FOUND => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::NotFoundError(message).into())
            }
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimitError.into()),
            _ => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::ApiError {
                    status: status.as_u16(),
                    message,
                }
                .into())
            }
        }
    }

    /// Make an authenticated GET request to v2 API
    async fn get_v2<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T> {
        let url = format!("{}/api/public/v2{}", self.host, path);

        let mut request = self
            .client
            .get(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key));

        if !params.is_empty() {
            request = request.query(params);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::TimeoutError
            } else {
                ApiError::NetworkError(e.to_string())
            }
        })?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let body = response
                    .json::<T>()
                    .await
                    .context("Failed to parse response")?;
                Ok(body)
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(ApiError::AuthenticationError.into())
            }
            StatusCode::NOT_FOUND => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::NotFoundError(message).into())
            }
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimitError.into()),
            _ => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::ApiError {
                    status: status.as_u16(),
                    message,
                }
                .into())
            }
        }
    }

    /// Make an authenticated POST request
    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/api/public{}", self.host, path);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
            .json(body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::TimeoutError
                } else {
                    ApiError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let body = response
                    .json::<T>()
                    .await
                    .context("Failed to parse response")?;
                Ok(body)
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(ApiError::AuthenticationError.into())
            }
            StatusCode::NOT_FOUND => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::NotFoundError(message).into())
            }
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimitError.into()),
            _ => {
                let message = response.text().await.unwrap_or_default();
                Err(ApiError::ApiError {
                    status: status.as_u16(),
                    message,
                }
                .into())
            }
        }
    }

    // ========== Traces API ==========

    /// List traces with optional filters
    #[allow(clippy::too_many_arguments)]
    pub async fn list_traces(
        &self,
        name: Option<&str>,
        user_id: Option<&str>,
        session_id: Option<&str>,
        tags: Option<&[String]>,
        from_timestamp: Option<&str>,
        to_timestamp: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<Trace>> {
        let mut all_traces = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(n) = name {
                params.push(("name", n.to_string()));
            }
            if let Some(u) = user_id {
                params.push(("userId", u.to_string()));
            }
            if let Some(s) = session_id {
                params.push(("sessionId", s.to_string()));
            }
            if let Some(from) = from_timestamp {
                params.push(("fromTimestamp", from.to_string()));
            }
            if let Some(to) = to_timestamp {
                params.push(("toTimestamp", to.to_string()));
            }
            if let Some(t) = tags {
                for tag in t {
                    params.push(("tags", tag.clone()));
                }
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: TracesResponse = self.get("/traces", &params_refs).await?;

            all_traces.extend(response.data);

            if all_traces.len() >= limit as usize {
                all_traces.truncate(limit as usize);
                break;
            }

            if let Some(meta) = &response.meta {
                if let Some(total_pages) = meta.total_pages {
                    if current_page >= total_pages as u32 {
                        break;
                    }
                }
            }

            current_page += 1;
        }

        Ok(all_traces)
    }

    /// Get a single trace by ID
    pub async fn get_trace(&self, id: &str) -> Result<Trace> {
        self.get(&format!("/traces/{id}"), &[]).await
    }

    // ========== Sessions API ==========

    /// List sessions with optional filters
    pub async fn list_sessions(
        &self,
        from_timestamp: Option<&str>,
        to_timestamp: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<Session>> {
        let mut all_sessions = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(from) = from_timestamp {
                params.push(("fromTimestamp", from.to_string()));
            }
            if let Some(to) = to_timestamp {
                params.push(("toTimestamp", to.to_string()));
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: SessionsResponse = self.get("/sessions", &params_refs).await?;

            all_sessions.extend(response.data);

            if all_sessions.len() >= limit as usize {
                all_sessions.truncate(limit as usize);
                break;
            }

            if let Some(meta) = &response.meta {
                if let Some(total_pages) = meta.total_pages {
                    if current_page >= total_pages as u32 {
                        break;
                    }
                }
            }

            current_page += 1;
        }

        Ok(all_sessions)
    }

    /// Get a single session by ID
    pub async fn get_session(&self, id: &str) -> Result<Session> {
        self.get(&format!("/sessions/{id}"), &[]).await
    }

    // ========== Observations API ==========

    /// List observations with optional filters
    #[allow(clippy::too_many_arguments)]
    pub async fn list_observations(
        &self,
        trace_id: Option<&str>,
        name: Option<&str>,
        observation_type: Option<&str>,
        user_id: Option<&str>,
        from_start_time: Option<&str>,
        to_start_time: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<Observation>> {
        let mut all_observations = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(t) = trace_id {
                params.push(("traceId", t.to_string()));
            }
            if let Some(n) = name {
                params.push(("name", n.to_string()));
            }
            if let Some(ot) = observation_type {
                params.push(("type", ot.to_string()));
            }
            if let Some(u) = user_id {
                params.push(("userId", u.to_string()));
            }
            if let Some(from) = from_start_time {
                params.push(("fromStartTime", from.to_string()));
            }
            if let Some(to) = to_start_time {
                params.push(("toStartTime", to.to_string()));
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: ObservationsResponse = self.get("/observations", &params_refs).await?;

            all_observations.extend(response.data);

            if all_observations.len() >= limit as usize {
                all_observations.truncate(limit as usize);
                break;
            }

            if let Some(meta) = &response.meta {
                if let Some(total_pages) = meta.total_pages {
                    if current_page >= total_pages as u32 {
                        break;
                    }
                }
            }

            current_page += 1;
        }

        Ok(all_observations)
    }

    /// Get a single observation by ID
    pub async fn get_observation(&self, id: &str) -> Result<Observation> {
        self.get(&format!("/observations/{id}"), &[]).await
    }

    // ========== Scores API ==========

    /// List scores with optional filters
    pub async fn list_scores(
        &self,
        name: Option<&str>,
        from_timestamp: Option<&str>,
        to_timestamp: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<Score>> {
        let mut all_scores = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(n) = name {
                params.push(("name", n.to_string()));
            }
            if let Some(from) = from_timestamp {
                params.push(("fromTimestamp", from.to_string()));
            }
            if let Some(to) = to_timestamp {
                params.push(("toTimestamp", to.to_string()));
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: ScoresResponse = self.get("/scores", &params_refs).await?;

            all_scores.extend(response.data);

            if all_scores.len() >= limit as usize {
                all_scores.truncate(limit as usize);
                break;
            }

            if let Some(meta) = &response.meta {
                if let Some(total_pages) = meta.total_pages {
                    if current_page >= total_pages as u32 {
                        break;
                    }
                }
            }

            current_page += 1;
        }

        Ok(all_scores)
    }

    /// Get a single score by ID
    pub async fn get_score(&self, id: &str) -> Result<Score> {
        self.get(&format!("/scores/{id}"), &[]).await
    }

    // ========== Metrics API ==========

    /// Query metrics
    #[allow(clippy::too_many_arguments)]
    pub async fn query_metrics(
        &self,
        view: &str,
        measure: &str,
        aggregation: &str,
        dimensions: Option<&[String]>,
        from_timestamp: Option<&str>,
        to_timestamp: Option<&str>,
        granularity: Option<&str>,
        limit: Option<u32>,
    ) -> Result<MetricsResult> {
        let mut body: HashMap<String, serde_json::Value> = HashMap::new();

        body.insert("view".to_string(), serde_json::json!(view));
        body.insert("measure".to_string(), serde_json::json!(measure));
        body.insert("aggregation".to_string(), serde_json::json!(aggregation));

        if let Some(dims) = dimensions {
            let dims_formatted: Vec<HashMap<String, String>> = dims
                .iter()
                .map(|d| {
                    let mut m = HashMap::new();
                    m.insert("field".to_string(), d.clone());
                    m
                })
                .collect();
            body.insert("dimensions".to_string(), serde_json::json!(dims_formatted));
        }

        if let Some(from) = from_timestamp {
            body.insert("fromTimestamp".to_string(), serde_json::json!(from));
        }
        if let Some(to) = to_timestamp {
            body.insert("toTimestamp".to_string(), serde_json::json!(to));
        }
        if let Some(g) = granularity {
            body.insert("granularity".to_string(), serde_json::json!(g));
        }
        if let Some(l) = limit {
            body.insert("limit".to_string(), serde_json::json!(l));
        }

        self.post("/metrics", &body).await
    }

    /// Test connectivity (used for config validation)
    pub async fn test_connection(&self) -> Result<bool> {
        // Try to list traces with limit 1 to test connection
        let params = [("limit", "1")];
        let result: Result<TracesResponse> = self.get("/traces", &params).await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    // ========== Prompts API ==========

    /// List prompts with optional filters
    pub async fn list_prompts(
        &self,
        name: Option<&str>,
        label: Option<&str>,
        tag: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<PromptMeta>> {
        let mut all_prompts = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(n) = name {
                params.push(("name", n.to_string()));
            }
            if let Some(l) = label {
                params.push(("label", l.to_string()));
            }
            if let Some(t) = tag {
                params.push(("tag", t.to_string()));
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: PromptsResponse = self.get_v2("/prompts", &params_refs).await?;

            all_prompts.extend(response.data);

            if all_prompts.len() >= limit as usize {
                all_prompts.truncate(limit as usize);
                break;
            }

            if let Some(meta) = &response.meta {
                if let Some(total_pages) = meta.total_pages {
                    if current_page >= total_pages as u32 {
                        break;
                    }
                }
            }

            current_page += 1;
        }

        Ok(all_prompts)
    }

    /// Get a specific prompt by name
    pub async fn get_prompt(
        &self,
        name: &str,
        version: Option<i32>,
        label: Option<&str>,
    ) -> Result<Prompt> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(v) = version {
            params.push(("version", v.to_string()));
        }
        if let Some(l) = label {
            params.push(("label", l.to_string()));
        }

        let params_refs: Vec<(&str, &str)> =
            params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        self.get_v2(&format!("/prompts/{}", name), &params_refs).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config(host: &str) -> Config {
        Config {
            public_key: Some("pk-test-123".to_string()),
            secret_key: Some("sk-test-456".to_string()),
            host: host.to_string(),
            profile: "test".to_string(),
            format: crate::types::OutputFormat::Table,
            limit: 50,
            page: 1,
            output: None,
            verbose: false,
            no_color: false,
        }
    }

    // ========== Client Creation Tests ==========

    #[test]
    fn test_client_new_with_valid_config() {
        let config = Config {
            public_key: Some("pk-test".to_string()),
            secret_key: Some("sk-test".to_string()),
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        let client = LangfuseClient::new(&config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_new_missing_public_key() {
        let config = Config {
            public_key: None,
            secret_key: Some("sk-test".to_string()),
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        let client = LangfuseClient::new(&config);
        assert!(client.is_err());
        assert!(client.unwrap_err().to_string().contains("Public key"));
    }

    #[test]
    fn test_client_new_missing_secret_key() {
        let config = Config {
            public_key: Some("pk-test".to_string()),
            secret_key: None,
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        let client = LangfuseClient::new(&config);
        assert!(client.is_err());
        assert!(client.unwrap_err().to_string().contains("Secret key"));
    }

    // ========== API Error Tests ==========

    #[test]
    fn test_api_error_display() {
        let auth_err = ApiError::AuthenticationError;
        assert!(auth_err.to_string().contains("Authentication failed"));

        let not_found = ApiError::NotFoundError("trace-123".to_string());
        assert!(not_found.to_string().contains("trace-123"));

        let rate_limit = ApiError::RateLimitError;
        assert!(rate_limit.to_string().contains("Rate limit"));

        let timeout = ApiError::TimeoutError;
        assert!(timeout.to_string().contains("timeout"));

        let api_err = ApiError::ApiError {
            status: 500,
            message: "Internal error".to_string(),
        };
        assert!(api_err.to_string().contains("500"));
        assert!(api_err.to_string().contains("Internal error"));

        let network_err = ApiError::NetworkError("Connection refused".to_string());
        assert!(network_err.to_string().contains("Connection refused"));
    }

    // ========== Traces API Tests ==========

    #[tokio::test]
    async fn test_list_traces_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .and(query_param("limit", "50"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "trace-1", "name": "Test Trace 1"},
                    {"id": "trace-2", "name": "Test Trace 2"}
                ],
                "meta": {
                    "page": 1,
                    "limit": 50,
                    "totalItems": 2,
                    "totalPages": 1
                }
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let traces = client
            .list_traces(None, None, None, None, None, None, 50, 1)
            .await
            .unwrap();

        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].id, "trace-1");
        assert_eq!(traces[1].id, "trace-2");
    }

    #[tokio::test]
    async fn test_list_traces_with_filters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .and(query_param("name", "my-trace"))
            .and(query_param("userId", "user-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{"id": "trace-1"}],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let traces = client
            .list_traces(
                Some("my-trace"),
                Some("user-123"),
                None,
                None,
                None,
                None,
                50,
                1,
            )
            .await
            .unwrap();

        assert_eq!(traces.len(), 1);
    }

    #[tokio::test]
    async fn test_get_trace_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces/trace-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "trace-123",
                "name": "My Trace",
                "userId": "user-456"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let trace = client.get_trace("trace-123").await.unwrap();

        assert_eq!(trace.id, "trace-123");
        assert_eq!(trace.name, Some("My Trace".to_string()));
    }

    #[tokio::test]
    async fn test_get_trace_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.get_trace("nonexistent").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("Not found"));
    }

    // ========== Sessions API Tests ==========

    #[tokio::test]
    async fn test_list_sessions_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/sessions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "session-1", "createdAt": "2024-01-15T10:00:00Z"},
                    {"id": "session-2", "createdAt": "2024-01-16T10:00:00Z"}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let sessions = client.list_sessions(None, None, 50, 1).await.unwrap();

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "session-1");
    }

    #[tokio::test]
    async fn test_get_session_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/sessions/session-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "session-123",
                "createdAt": "2024-01-15T10:00:00Z",
                "projectId": "project-456"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let session = client.get_session("session-123").await.unwrap();

        assert_eq!(session.id, "session-123");
        assert_eq!(session.project_id, Some("project-456".to_string()));
    }

    // ========== Observations API Tests ==========

    #[tokio::test]
    async fn test_list_observations_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/observations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "obs-1", "type": "GENERATION", "name": "completion"},
                    {"id": "obs-2", "type": "SPAN", "name": "process"}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let observations = client
            .list_observations(None, None, None, None, None, None, 50, 1)
            .await
            .unwrap();

        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].r#type, Some("GENERATION".to_string()));
    }

    #[tokio::test]
    async fn test_list_observations_with_trace_filter() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/observations"))
            .and(query_param("traceId", "trace-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{"id": "obs-1", "traceId": "trace-123"}],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let observations = client
            .list_observations(Some("trace-123"), None, None, None, None, None, 50, 1)
            .await
            .unwrap();

        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].trace_id, Some("trace-123".to_string()));
    }

    #[tokio::test]
    async fn test_get_observation_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/observations/obs-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "obs-123",
                "type": "GENERATION",
                "model": "gpt-4"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let observation = client.get_observation("obs-123").await.unwrap();

        assert_eq!(observation.id, "obs-123");
        assert_eq!(observation.model, Some("gpt-4".to_string()));
    }

    // ========== Scores API Tests ==========

    #[tokio::test]
    async fn test_list_scores_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/scores"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "score-1", "name": "accuracy", "value": 0.95},
                    {"id": "score-2", "name": "relevance", "value": 0.88}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let scores = client.list_scores(None, None, None, 50, 1).await.unwrap();

        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].name, Some("accuracy".to_string()));
    }

    #[tokio::test]
    async fn test_get_score_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/scores/score-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "score-123",
                "name": "accuracy",
                "value": 0.95,
                "source": "API"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let score = client.get_score("score-123").await.unwrap();

        assert_eq!(score.id, "score-123");
        assert_eq!(score.source, Some("API".to_string()));
    }

    // ========== Metrics API Tests ==========

    #[tokio::test]
    async fn test_query_metrics_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"timestamp": "2024-01-15", "count": 100},
                    {"timestamp": "2024-01-16", "count": 150}
                ]
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .query_metrics("traces", "count", "count", None, None, None, None, None)
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
    }

    #[tokio::test]
    async fn test_query_metrics_with_dimensions() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"model": "gpt-4", "count": 50},
                    {"model": "gpt-3.5", "count": 100}
                ]
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let dimensions = vec!["model".to_string()];
        let result = client
            .query_metrics(
                "observations",
                "count",
                "count",
                Some(&dimensions),
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.data.len(), 2);
    }

    // ========== Authentication Tests ==========

    #[tokio::test]
    async fn test_authentication_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .list_traces(None, None, None, None, None, None, 50, 1)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Authentication failed"));
    }

    #[tokio::test]
    async fn test_forbidden_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .list_traces(None, None, None, None, None, None, 50, 1)
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Authentication failed"));
    }

    // ========== Rate Limiting Tests ==========

    #[tokio::test]
    async fn test_rate_limit_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .list_traces(None, None, None, None, None, None, 50, 1)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rate limit"));
    }

    // ========== Server Error Tests ==========

    #[tokio::test]
    async fn test_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .list_traces(None, None, None, None, None, None, 50, 1)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("500"));
    }

    // ========== Test Connection Tests ==========

    #[tokio::test]
    async fn test_connection_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .and(query_param("limit", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [],
                "meta": {}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.test_connection().await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_connection_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.test_connection().await;

        assert!(result.is_err());
    }

    // ========== Prompts API Tests ==========

    #[tokio::test]
    async fn test_list_prompts_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts"))
            .and(query_param("limit", "50"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"name": "prompt-1", "versions": [1, 2], "labels": ["production"], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"},
                    {"name": "prompt-2", "versions": [1], "labels": [], "tags": ["test"], "lastUpdatedAt": "2024-01-15T10:00:00Z"}
                ],
                "meta": {
                    "page": 1,
                    "limit": 50,
                    "totalItems": 2,
                    "totalPages": 1
                }
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompts = client
            .list_prompts(None, None, None, 50, 1)
            .await
            .unwrap();

        assert_eq!(prompts.len(), 2);
        assert_eq!(prompts[0].name, "prompt-1");
        assert_eq!(prompts[0].versions, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_list_prompts_with_filters() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts"))
            .and(query_param("name", "welcome"))
            .and(query_param("label", "production"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{"name": "welcome", "versions": [1], "labels": ["production"], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"}],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompts = client
            .list_prompts(Some("welcome"), Some("production"), None, 50, 1)
            .await
            .unwrap();

        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "welcome");
    }

    #[tokio::test]
    async fn test_get_prompt_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts/welcome"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "welcome",
                "version": 2,
                "type": "text",
                "prompt": "Hello {{name}}!",
                "labels": ["production"],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompt = client.get_prompt("welcome", None, None).await.unwrap();

        assert_eq!(prompt.name, "welcome");
        assert_eq!(prompt.version, 2);
    }

    #[tokio::test]
    async fn test_get_prompt_with_version() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts/welcome"))
            .and(query_param("version", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "welcome",
                "version": 1,
                "type": "text",
                "prompt": "Hi {{name}}!",
                "labels": [],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompt = client.get_prompt("welcome", Some(1), None).await.unwrap();

        assert_eq!(prompt.version, 1);
    }

    // ========== Pagination Tests ==========

    #[tokio::test]
    async fn test_list_traces_pagination() {
        let mock_server = MockServer::start().await;

        // First page - server returns only 2 items despite our limit=3
        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .and(query_param("page", "1"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "trace-1"},
                    {"id": "trace-2"}
                ],
                "meta": {
                    "page": 1,
                    "totalPages": 2
                }
            })))
            .mount(&mock_server)
            .await;

        // Second page
        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .and(query_param("page", "2"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "trace-3"}
                ],
                "meta": {
                    "page": 2,
                    "totalPages": 2
                }
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        // Request 3 items, should fetch both pages
        let traces = client
            .list_traces(None, None, None, None, None, None, 3, 1)
            .await
            .unwrap();

        assert_eq!(traces.len(), 3);
        assert_eq!(traces[0].id, "trace-1");
        assert_eq!(traces[2].id, "trace-3");
    }

    #[tokio::test]
    async fn test_list_traces_limit_truncation() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/traces"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "trace-1"},
                    {"id": "trace-2"},
                    {"id": "trace-3"}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        // Request only 2 items
        let traces = client
            .list_traces(None, None, None, None, None, None, 2, 1)
            .await
            .unwrap();

        assert_eq!(traces.len(), 2);
    }
}
