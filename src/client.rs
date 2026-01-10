use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use thiserror::Error;

use crate::config::Config;
use crate::types::*;

/// API errors
#[derive(Error, Debug)]
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

        let mut request = self.client.get(&url).basic_auth(&self.public_key, Some(&self.secret_key));

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
        self.get(&format!("/traces/{}", id), &[]).await
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
        self.get(&format!("/sessions/{}", id), &[]).await
    }

    // ========== Observations API ==========

    /// List observations with optional filters
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
        self.get(&format!("/observations/{}", id), &[]).await
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
        self.get(&format!("/scores/{}", id), &[]).await
    }

    // ========== Metrics API ==========

    /// Query metrics
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
}
