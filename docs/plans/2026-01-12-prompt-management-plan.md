# Prompt Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add full CRUD operations for Langfuse prompts (list, get, create-text, create-chat, label, delete).

**Architecture:** Follow existing patterns - types in types.rs, client methods in client.rs, command module in commands/prompts.rs, wire up in main.rs. API uses `/api/public/v2/prompts` endpoints.

**Tech Stack:** Rust, clap for CLI, reqwest for HTTP, serde for JSON, wiremock for tests.

---

## Task 1: Add Prompt Types

**Files:**
- Modify: `src/types.rs`

**Step 1: Write tests for prompt types**

Add to the `#[cfg(test)]` module at the end of `src/types.rs`:

```rust
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test prompt --lib`
Expected: FAIL - types don't exist yet

**Step 3: Add the type definitions**

Add before the `#[cfg(test)]` module in `src/types.rs`:

```rust
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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test prompt --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/types.rs
git commit -m "feat(prompts): add prompt types"
```

---

## Task 2: Add Client Methods for List and Get

**Files:**
- Modify: `src/client.rs`

**Step 1: Write tests for list_prompts**

Add to the `#[cfg(test)]` module in `src/client.rs`:

```rust
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test prompts --lib`
Expected: FAIL - methods don't exist yet

**Step 3: Add list_prompts and get_prompt methods**

Add a new `get_v2` helper method and the prompts methods in `src/client.rs` after the Metrics API section:

```rust
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
```

Also add the import at the top of client.rs in the `use crate::types::*;` section (it already imports all types).

**Step 4: Run tests to verify they pass**

Run: `cargo test prompts --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/client.rs
git commit -m "feat(prompts): add list and get client methods"
```

---

## Task 3: Add Client Methods for Create, Update Labels, Delete

**Files:**
- Modify: `src/client.rs`

**Step 1: Write tests for create, update, delete**

Add to the `#[cfg(test)]` module in `src/client.rs`:

```rust
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
        .create_chat_prompt("assistant", &messages, None, None, None)
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test test_create --lib && cargo test test_update_prompt --lib && cargo test test_delete_prompt --lib`
Expected: FAIL - methods don't exist yet

**Step 3: Add the methods**

Add to `src/client.rs` after `get_prompt`:

```rust
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

    /// Create a text prompt
    pub async fn create_text_prompt(
        &self,
        name: &str,
        prompt: &str,
        labels: Option<&[String]>,
        tags: Option<&[String]>,
        config: Option<&serde_json::Value>,
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
            "labels": labels,
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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test prompts --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/client.rs
git commit -m "feat(prompts): add create, update, delete client methods"
```

---

## Task 4: Create Prompts Command Module

**Files:**
- Create: `src/commands/prompts.rs`

**Step 1: Create the prompts command file**

Create `src/commands/prompts.rs`:

```rust
// ABOUTME: Command handlers for prompt management operations
// ABOUTME: Supports list, get, create-text, create-chat, label, and delete

use anyhow::Result;
use clap::Subcommand;
use std::io::{self, Read};

use crate::client::LangfuseClient;
use crate::commands::{build_config, format_and_output, output_result};
use crate::types::{ChatMessage, OutputFormat, PromptContent};

#[derive(Debug, Subcommand)]
pub enum PromptsCommands {
    /// List prompts with optional filters
    List {
        /// Filter by prompt name
        #[arg(short, long)]
        name: Option<String>,

        /// Filter by label
        #[arg(short, long)]
        label: Option<String>,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: u32,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get a specific prompt by name
    Get {
        /// Prompt name
        name: String,

        /// Specific version number
        #[arg(long)]
        version: Option<i32>,

        /// Fetch by label (default: production)
        #[arg(short, long)]
        label: Option<String>,

        /// Output raw content only (for piping)
        #[arg(long)]
        raw: bool,

        /// Output format (ignored if --raw)
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Create a text prompt
    CreateText {
        /// Prompt name
        #[arg(long)]
        name: String,

        /// Read content from file (reads stdin if omitted)
        #[arg(short, long)]
        file: Option<String>,

        /// Labels to apply
        #[arg(short, long)]
        labels: Option<Vec<String>>,

        /// Tags to apply
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Model config as JSON string
        #[arg(long)]
        config: Option<String>,

        /// Output format
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Create a chat prompt
    CreateChat {
        /// Prompt name
        #[arg(long)]
        name: String,

        /// Read JSON messages from file (reads stdin if omitted)
        #[arg(short, long)]
        file: Option<String>,

        /// Labels to apply
        #[arg(short, long)]
        labels: Option<Vec<String>>,

        /// Tags to apply
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Model config as JSON string
        #[arg(long)]
        config: Option<String>,

        /// Output format
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Set labels on a prompt version
    Label {
        /// Prompt name
        name: String,

        /// Version number
        version: i32,

        /// Labels to set
        #[arg(short, long, required = true)]
        labels: Vec<String>,

        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Delete a prompt
    Delete {
        /// Prompt name
        name: String,

        /// Delete specific version only
        #[arg(long)]
        version: Option<i32>,

        /// Delete versions with this label only
        #[arg(short, long)]
        label: Option<String>,

        /// Profile name
        #[arg(long)]
        profile: Option<String>,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: Option<String>,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: Option<String>,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn read_content(file: Option<&str>) -> Result<String> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

impl PromptsCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            PromptsCommands::List {
                name,
                label,
                tag,
                limit,
                page,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    Some(*limit),
                    Some(*page),
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                let prompts = client
                    .list_prompts(
                        name.as_deref(),
                        label.as_deref(),
                        tag.as_deref(),
                        *limit,
                        *page,
                    )
                    .await?;

                format_and_output(
                    &prompts,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Get {
                name,
                version,
                label,
                raw,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    None,
                    None,
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                let prompt = client
                    .get_prompt(name, *version, label.as_deref())
                    .await?;

                if *raw {
                    let content = match &prompt.prompt {
                        PromptContent::Text(s) => s.clone(),
                        PromptContent::Chat(msgs) => serde_json::to_string_pretty(msgs)?,
                    };
                    output_result(&content, output.as_deref(), *verbose)
                } else {
                    format_and_output(
                        &prompt,
                        format.unwrap_or(OutputFormat::Table),
                        output.as_deref(),
                        *verbose,
                    )
                }
            }

            PromptsCommands::CreateText {
                name,
                file,
                labels,
                tags,
                config: cfg,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let app_config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    None,
                    None,
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !app_config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let content = read_content(file.as_deref())?;
                let parsed_config: Option<serde_json::Value> = cfg
                    .as_ref()
                    .map(|c| serde_json::from_str(c))
                    .transpose()?;

                let client = LangfuseClient::new(&app_config)?;

                let prompt = client
                    .create_text_prompt(
                        name,
                        &content,
                        labels.as_deref(),
                        tags.as_deref(),
                        parsed_config.as_ref(),
                    )
                    .await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::CreateChat {
                name,
                file,
                labels,
                tags,
                config: cfg,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let app_config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    None,
                    None,
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !app_config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let content = read_content(file.as_deref())?;
                let messages: Vec<ChatMessage> = serde_json::from_str(&content)?;
                let parsed_config: Option<serde_json::Value> = cfg
                    .as_ref()
                    .map(|c| serde_json::from_str(c))
                    .transpose()?;

                let client = LangfuseClient::new(&app_config)?;

                let prompt = client
                    .create_chat_prompt(
                        name,
                        &messages,
                        labels.as_deref(),
                        tags.as_deref(),
                        parsed_config.as_ref(),
                    )
                    .await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Label {
                name,
                version,
                labels,
                format,
                output,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    *format,
                    None,
                    None,
                    output.as_deref(),
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                let prompt = client
                    .update_prompt_labels(name, *version, labels)
                    .await?;

                format_and_output(
                    &prompt,
                    format.unwrap_or(OutputFormat::Table),
                    output.as_deref(),
                    *verbose,
                )
            }

            PromptsCommands::Delete {
                name,
                version,
                label,
                profile,
                public_key,
                secret_key,
                host,
                verbose,
            } => {
                let config = build_config(
                    profile.as_deref(),
                    public_key.as_deref(),
                    secret_key.as_deref(),
                    host.as_deref(),
                    None,
                    None,
                    None,
                    None,
                    *verbose,
                    false,
                )?;

                if !config.is_valid() {
                    eprintln!("Error: Missing credentials. Run 'lf config setup' or set environment variables.");
                    std::process::exit(1);
                }

                let client = LangfuseClient::new(&config)?;

                client
                    .delete_prompt(name, *version, label.as_deref())
                    .await?;

                if *verbose {
                    eprintln!("Prompt '{}' deleted successfully", name);
                }

                Ok(())
            }
        }
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Errors about module not being declared

**Step 3: Commit**

```bash
git add src/commands/prompts.rs
git commit -m "feat(prompts): add prompts command module"
```

---

## Task 5: Wire Up Commands

**Files:**
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`

**Step 1: Add module export to mod.rs**

Add to `src/commands/mod.rs` after line 6:

```rust
pub mod prompts;
```

**Step 2: Add import and command variant to main.rs**

In `src/main.rs`, add after line 15:

```rust
use commands::prompts::PromptsCommands;
```

Add to the `Commands` enum after `Metrics`:

```rust
    /// Manage prompts
    #[command(subcommand)]
    Prompts(PromptsCommands),
```

Add to the match statement after `Commands::Metrics`:

```rust
        Commands::Prompts(cmd) => cmd.execute().await,
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: PASS

**Step 4: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/mod.rs src/main.rs
git commit -m "feat(prompts): wire up prompts commands to CLI"
```

---

## Task 6: Manual Verification

**Step 1: Build release binary**

Run: `cargo build --release`

**Step 2: Test help output**

Run: `./target/release/lf prompts --help`
Expected: Shows list, get, create-text, create-chat, label, delete subcommands

Run: `./target/release/lf prompts list --help`
Expected: Shows all filter and output options

Run: `./target/release/lf prompts create-text --help`
Expected: Shows --name, --file, --labels, --tags, --config options

**Step 3: Commit final state**

```bash
git add -A
git commit -m "feat(prompts): complete prompt management implementation"
```
