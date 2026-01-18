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

    /// Make an authenticated POST request to v2 API
    async fn post_v2<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/api/public/v2{}", self.host, path);

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
            StatusCode::OK | StatusCode::CREATED => {
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

    /// Make an authenticated PATCH request to v2 API
    async fn patch_v2<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}/api/public/v2{}", self.host, path);

        let response = self
            .client
            .patch(&url)
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

    /// Make an authenticated DELETE request to v2 API
    async fn delete_v2(&self, path: &str, params: &[(&str, &str)]) -> Result<()> {
        let url = format!("{}/api/public/v2{}", self.host, path);

        let mut request = self
            .client
            .delete(&url)
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
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
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
            StatusCode::OK | StatusCode::CREATED => {
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

    /// Create a new score
    #[allow(clippy::too_many_arguments)]
    pub async fn create_score(
        &self,
        name: &str,
        value: f64,
        trace_id: Option<&str>,
        observation_id: Option<&str>,
        session_id: Option<&str>,
        data_type: Option<&str>,
        comment: Option<&str>,
    ) -> Result<CreateScoreResponse> {
        let mut body = serde_json::json!({
            "name": name,
            "value": value,
        });

        if let Some(tid) = trace_id {
            body["traceId"] = serde_json::json!(tid);
        }
        if let Some(oid) = observation_id {
            body["observationId"] = serde_json::json!(oid);
        }
        if let Some(sid) = session_id {
            body["sessionId"] = serde_json::json!(sid);
        }
        if let Some(dt) = data_type {
            body["dataType"] = serde_json::json!(dt);
        }
        if let Some(c) = comment {
            body["comment"] = serde_json::json!(c);
        }

        self.post("/scores", &body).await
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

        self.get_v2(&format!("/prompts/{}", name), &params_refs)
            .await
    }

    /// Create a text prompt
    pub async fn create_text_prompt(
        &self,
        name: &str,
        prompt: &str,
        labels: Option<&[String]>,
        tags: Option<&[String]>,
        config: Option<&serde_json::Value>,
        commit_message: Option<&str>,
    ) -> Result<Prompt> {
        let mut body = serde_json::json!({
            "name": name,
            "type": "text",
            "prompt": prompt,
        });

        if let Some(l) = labels {
            body["labels"] = serde_json::json!(l);
        }
        if let Some(t) = tags {
            body["tags"] = serde_json::json!(t);
        }
        if let Some(c) = config {
            body["config"] = c.clone();
        }
        if let Some(m) = commit_message {
            body["commitMessage"] = serde_json::json!(m);
        }

        self.post_v2("/prompts", &body).await
    }

    /// Create a chat prompt
    pub async fn create_chat_prompt(
        &self,
        name: &str,
        messages: &[ChatMessage],
        labels: Option<&[String]>,
        tags: Option<&[String]>,
        config: Option<&serde_json::Value>,
        commit_message: Option<&str>,
    ) -> Result<Prompt> {
        let mut body = serde_json::json!({
            "name": name,
            "type": "chat",
            "prompt": messages,
        });

        if let Some(l) = labels {
            body["labels"] = serde_json::json!(l);
        }
        if let Some(t) = tags {
            body["tags"] = serde_json::json!(t);
        }
        if let Some(c) = config {
            body["config"] = c.clone();
        }
        if let Some(m) = commit_message {
            body["commitMessage"] = serde_json::json!(m);
        }

        self.post_v2("/prompts", &body).await
    }

    /// Update labels on a prompt version
    pub async fn update_prompt_labels(
        &self,
        name: &str,
        version: i32,
        labels: &[String],
    ) -> Result<Prompt> {
        let body = serde_json::json!({
            "newLabels": labels,
        });

        self.patch_v2(&format!("/prompts/{}/versions/{}", name, version), &body)
            .await
    }

    /// Delete a prompt (or specific version/label)
    pub async fn delete_prompt(
        &self,
        name: &str,
        version: Option<i32>,
        label: Option<&str>,
    ) -> Result<()> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(v) = version {
            params.push(("version", v.to_string()));
        }
        if let Some(l) = label {
            params.push(("label", l.to_string()));
        }

        let params_refs: Vec<(&str, &str)> =
            params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        self.delete_v2(&format!("/prompts/{}", name), &params_refs)
            .await
    }

    // ========== Datasets API ==========

    /// List datasets with optional pagination
    pub async fn list_datasets(&self, limit: u32, page: u32) -> Result<Vec<Dataset>> {
        let mut all_datasets = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: DatasetsResponse = self.get_v2("/datasets", &params_refs).await?;

            all_datasets.extend(response.data);

            if all_datasets.len() >= limit as usize {
                all_datasets.truncate(limit as usize);
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

        Ok(all_datasets)
    }

    /// Get a dataset by name
    pub async fn get_dataset(&self, name: &str) -> Result<Dataset> {
        self.get_v2(&format!("/datasets/{}", name), &[]).await
    }

    /// Create a new dataset
    pub async fn create_dataset(
        &self,
        name: &str,
        description: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> Result<Dataset> {
        let mut body = serde_json::json!({
            "name": name,
        });

        if let Some(d) = description {
            body["description"] = serde_json::json!(d);
        }
        if let Some(m) = metadata {
            body["metadata"] = m.clone();
        }

        self.post_v2("/datasets", &body).await
    }

    // ========== Dataset Items API ==========

    /// List dataset items with optional filters
    pub async fn list_dataset_items(
        &self,
        dataset_name: Option<&str>,
        limit: u32,
        page: u32,
    ) -> Result<Vec<DatasetItem>> {
        let mut all_items = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            if let Some(name) = dataset_name {
                params.push(("datasetName", name.to_string()));
            }

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: DatasetItemsResponse = self.get("/dataset-items", &params_refs).await?;

            all_items.extend(response.data);

            if all_items.len() >= limit as usize {
                all_items.truncate(limit as usize);
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

        Ok(all_items)
    }

    /// Get a dataset item by ID
    pub async fn get_dataset_item(&self, id: &str) -> Result<DatasetItem> {
        self.get(&format!("/dataset-items/{}", id), &[]).await
    }

    /// Create a dataset item
    pub async fn create_dataset_item(
        &self,
        dataset_name: &str,
        input: &serde_json::Value,
        expected_output: Option<&serde_json::Value>,
        metadata: Option<&serde_json::Value>,
        source_trace_id: Option<&str>,
        source_observation_id: Option<&str>,
    ) -> Result<DatasetItem> {
        let mut body = serde_json::json!({
            "datasetName": dataset_name,
            "input": input,
        });

        if let Some(eo) = expected_output {
            body["expectedOutput"] = eo.clone();
        }
        if let Some(m) = metadata {
            body["metadata"] = m.clone();
        }
        if let Some(tid) = source_trace_id {
            body["sourceTraceId"] = serde_json::json!(tid);
        }
        if let Some(oid) = source_observation_id {
            body["sourceObservationId"] = serde_json::json!(oid);
        }

        self.post("/dataset-items", &body).await
    }

    // ========== Dataset Runs API ==========

    /// List dataset runs for a dataset
    pub async fn list_dataset_runs(
        &self,
        dataset_name: &str,
        limit: u32,
        page: u32,
    ) -> Result<Vec<DatasetRun>> {
        let mut all_runs = Vec::new();
        let mut current_page = page;
        let page_size = std::cmp::min(limit, 100);

        loop {
            let params: Vec<(&str, String)> = vec![
                ("limit", page_size.to_string()),
                ("page", current_page.to_string()),
            ];

            let params_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let response: DatasetRunsResponse = self
                .get(&format!("/datasets/{}/runs", dataset_name), &params_refs)
                .await?;

            all_runs.extend(response.data);

            if all_runs.len() >= limit as usize {
                all_runs.truncate(limit as usize);
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

        Ok(all_runs)
    }

    /// Get a dataset run by name
    pub async fn get_dataset_run(&self, dataset_name: &str, run_name: &str) -> Result<DatasetRun> {
        self.get(&format!("/datasets/{}/runs/{}", dataset_name, run_name), &[])
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param};
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

    // ========== Prompts Create/Update/Delete Tests ==========

    #[tokio::test]
    async fn test_create_text_prompt_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/v2/prompts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "greeting",
                "version": 1,
                "type": "text",
                "prompt": "Hello {{name}}!",
                "labels": ["staging"],
                "tags": ["test"],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompt = client
            .create_text_prompt(
                "greeting",
                "Hello {{name}}!",
                Some(&["staging".to_string()]),
                Some(&["test".to_string()]),
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(prompt.name, "greeting");
        assert_eq!(prompt.version, 1);
    }

    #[tokio::test]
    async fn test_create_chat_prompt_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/v2/prompts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "assistant",
                "version": 1,
                "type": "chat",
                "prompt": [{"role": "system", "content": "You are helpful."}],
                "labels": [],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let messages = vec![ChatMessage {
            role: "system".to_string(),
            content: "You are helpful.".to_string(),
        }];

        let prompt = client
            .create_chat_prompt("assistant", &messages, None, None, None, None)
            .await
            .unwrap();

        assert_eq!(prompt.name, "assistant");
        assert_eq!(prompt.prompt_type, "chat");
    }

    #[tokio::test]
    async fn test_update_prompt_labels_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/public/v2/prompts/greeting/versions/2"))
            .and(body_json(json!({
                "newLabels": ["production"]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "greeting",
                "version": 2,
                "type": "text",
                "prompt": "Hello!",
                "labels": ["production"],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompt = client
            .update_prompt_labels("greeting", 2, &["production".to_string()])
            .await
            .unwrap();

        assert_eq!(prompt.labels, vec!["production"]);
    }

    #[tokio::test]
    async fn test_delete_prompt_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/public/v2/prompts/greeting"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.delete_prompt("greeting", None, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_prompt_with_version() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/public/v2/prompts/greeting"))
            .and(query_param("version", "1"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.delete_prompt("greeting", Some(1), None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_prompt_with_label() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts/welcome"))
            .and(query_param("label", "staging"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "welcome",
                "version": 2,
                "type": "text",
                "prompt": "Staging version",
                "labels": ["staging"],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let prompt = client
            .get_prompt("welcome", None, Some("staging"))
            .await
            .unwrap();

        assert_eq!(prompt.labels, vec!["staging"]);
    }

    #[tokio::test]
    async fn test_delete_prompt_with_label() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/public/v2/prompts/greeting"))
            .and(query_param("label", "staging"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .delete_prompt("greeting", None, Some("staging"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_prompts_pagination() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts"))
            .and(query_param("page", "1"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"name": "prompt-1", "versions": [1], "labels": [], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"},
                    {"name": "prompt-2", "versions": [1], "labels": [], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"}
                ],
                "meta": {
                    "page": 1,
                    "totalPages": 2
                }
            })))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts"))
            .and(query_param("page", "2"))
            .and(query_param("limit", "3"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"name": "prompt-3", "versions": [1], "labels": [], "tags": [], "lastUpdatedAt": "2024-01-15T10:00:00Z"}
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

        let prompts = client.list_prompts(None, None, None, 3, 1).await.unwrap();

        assert_eq!(prompts.len(), 3);
        assert_eq!(prompts[0].name, "prompt-1");
        assert_eq!(prompts[2].name, "prompt-3");
    }

    #[tokio::test]
    async fn test_get_prompt_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Prompt not found"))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.get_prompt("nonexistent", None, None).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(err_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_list_prompts_auth_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/prompts"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client.list_prompts(None, None, None, 50, 1).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Authentication failed"));
    }

    #[tokio::test]
    async fn test_create_prompt_rate_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/v2/prompts"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .create_text_prompt("test", "content", None, None, None, None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Rate limit"));
    }

    // ========== Score Creation Tests ==========

    #[tokio::test]
    async fn test_create_score_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/scores"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "score-abc123"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .create_score(
                "accuracy",
                0.95,
                Some("trace-123"),
                None,
                None,
                Some("NUMERIC"),
                Some("Good result"),
            )
            .await
            .unwrap();

        assert_eq!(result.id, "score-abc123");
    }

    #[tokio::test]
    async fn test_create_score_with_observation() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/scores"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "score-def456"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .create_score(
                "relevance",
                0.88,
                Some("trace-123"),
                Some("obs-456"),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.id, "score-def456");
    }

    #[tokio::test]
    async fn test_create_score_handles_201_created() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/scores"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "score-new"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .create_score("test", 1.0, Some("trace-1"), None, None, None, None)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "score-new");
    }

    #[tokio::test]
    async fn test_create_prompt_handles_201_created() {
        let mock_server = MockServer::start().await;

        // API returns 201 Created for successful resource creation
        Mock::given(method("POST"))
            .and(path("/api/public/v2/prompts"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "name": "test-prompt",
                "version": 1,
                "type": "text",
                "prompt": "Test content",
                "labels": [],
                "tags": [],
                "createdAt": "2024-01-15T10:00:00Z",
                "updatedAt": "2024-01-15T10:00:00Z"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let result = client
            .create_text_prompt("test-prompt", "Test content", None, None, None, None)
            .await;

        assert!(result.is_ok(), "201 Created should be treated as success");
        let prompt = result.unwrap();
        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.version, 1);
    }

    // ========== Datasets API Tests ==========

    #[tokio::test]
    async fn test_list_datasets_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/datasets"))
            .and(query_param("limit", "50"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "ds-1", "name": "dataset-1", "description": "First dataset"},
                    {"id": "ds-2", "name": "dataset-2"}
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

        let datasets = client.list_datasets(50, 1).await.unwrap();

        assert_eq!(datasets.len(), 2);
        assert_eq!(datasets[0].name, "dataset-1");
        assert_eq!(datasets[1].name, "dataset-2");
    }

    #[tokio::test]
    async fn test_get_dataset_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/v2/datasets/my-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "ds-123",
                "name": "my-dataset",
                "description": "Test dataset",
                "projectId": "proj-456"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let dataset = client.get_dataset("my-dataset").await.unwrap();

        assert_eq!(dataset.id, "ds-123");
        assert_eq!(dataset.name, "my-dataset");
    }

    #[tokio::test]
    async fn test_create_dataset_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/v2/datasets"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "ds-new",
                "name": "new-dataset",
                "description": "A new dataset"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let dataset = client
            .create_dataset("new-dataset", Some("A new dataset"), None)
            .await
            .unwrap();

        assert_eq!(dataset.name, "new-dataset");
    }

    #[tokio::test]
    async fn test_list_dataset_items_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/dataset-items"))
            .and(query_param("datasetName", "my-dataset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "item-1", "datasetName": "my-dataset", "input": {"prompt": "Hello"}},
                    {"id": "item-2", "datasetName": "my-dataset", "input": {"prompt": "World"}}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let items = client
            .list_dataset_items(Some("my-dataset"), 50, 1)
            .await
            .unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "item-1");
    }

    #[tokio::test]
    async fn test_get_dataset_item_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/dataset-items/item-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "item-123",
                "datasetName": "my-dataset",
                "input": {"prompt": "Test"},
                "expectedOutput": {"response": "Expected"}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let item = client.get_dataset_item("item-123").await.unwrap();

        assert_eq!(item.id, "item-123");
    }

    #[tokio::test]
    async fn test_create_dataset_item_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/public/dataset-items"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "item-new",
                "datasetName": "my-dataset",
                "input": {"prompt": "New item"}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let input = json!({"prompt": "New item"});
        let item = client
            .create_dataset_item("my-dataset", &input, None, None, None, None)
            .await
            .unwrap();

        assert_eq!(item.id, "item-new");
    }

    #[tokio::test]
    async fn test_list_dataset_runs_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/datasets/my-dataset/runs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [
                    {"id": "run-1", "name": "eval-run-1", "datasetName": "my-dataset"},
                    {"id": "run-2", "name": "eval-run-2", "datasetName": "my-dataset"}
                ],
                "meta": {"totalPages": 1}
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let runs = client
            .list_dataset_runs("my-dataset", 50, 1)
            .await
            .unwrap();

        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].name, "eval-run-1");
    }

    #[tokio::test]
    async fn test_get_dataset_run_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/public/datasets/my-dataset/runs/eval-run"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "run-123",
                "name": "eval-run",
                "datasetName": "my-dataset",
                "description": "Evaluation run"
            })))
            .mount(&mock_server)
            .await;

        let config = create_test_config(&mock_server.uri());
        let client = LangfuseClient::new(&config).unwrap();

        let run = client
            .get_dataset_run("my-dataset", "eval-run")
            .await
            .unwrap();

        assert_eq!(run.id, "run-123");
        assert_eq!(run.name, "eval-run");
    }
}
