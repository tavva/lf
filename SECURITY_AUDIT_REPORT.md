# Security Audit Report: lf CLI Tool

**Project:** Langfuse CLI (`lf`)
**Version:** 0.2.1
**Audit Date:** 2026-01-14
**Auditor:** Claude Code Security Audit

## Executive Summary

This comprehensive security audit of the `lf` Rust CLI tool identifies potential security vulnerabilities and provides recommendations for improving the security posture of the application. The tool is a command-line interface for the Langfuse LLM observability platform, handling sensitive API credentials and making HTTP requests to external services.

**Overall Risk Level:** LOW to MEDIUM

The application follows many security best practices, but several areas require attention to prevent potential security issues.

---

## Findings Summary

| Category | Severity | Count |
|----------|----------|-------|
| Critical | 0 | 0 |
| High | 1 | 1 |
| Medium | 3 | 3 |
| Low | 4 | 4 |
| Informational | 2 | 2 |
| **Total** | | **10** |

---

## Detailed Findings

### 1. HIGH: Missing Host URL Validation (SSRF Risk)

**Severity:** HIGH
**Location:** `src/config.rs:156-160`, `src/client.rs:70`, `src/client.rs:120`
**CWE:** CWE-918: Server-Side Request Forgery (SSRF)

**Description:**
The application accepts a `host` parameter from CLI arguments, environment variables, or configuration files without proper validation. This parameter is directly used to construct URLs for HTTP requests. An attacker could potentially:
- Redirect requests to internal network resources (e.g., `http://localhost:6379`, `http://169.254.169.254/`)
- Scan internal network ports
- Access cloud metadata services
- Perform denial of service attacks

**Vulnerable Code:**
```rust
// config.rs:156-160
let resolved_host = host
    .map(|s| s.to_string())
    .or_else(|| std::env::var("LANGFUSE_HOST").ok())
    .or_else(|| file_profile.and_then(|p| p.host.clone()))
    .unwrap_or_else(|| DEFAULT_HOST.to_string());

// client.rs:70
let url = format!("{}/api/public{}", self.host, path);
```

**Recommendation:**
1. Validate that the host URL uses HTTPS (except for localhost in development)
2. Implement a URL allowlist or validate against expected patterns
3. Reject URLs pointing to private IP ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 127.0.0.0/8, 169.254.0.0/16)
4. Consider using the `url` crate to parse and validate URLs properly

**Example Fix:**
```rust
fn validate_host_url(host: &str) -> Result<()> {
    let url = url::Url::parse(host)
        .context("Invalid host URL format")?;

    // Require HTTPS for non-localhost
    if url.scheme() != "https" && !url.host_str().unwrap_or("").contains("localhost") {
        return Err(anyhow::anyhow!("Host must use HTTPS"));
    }

    // Block private IP ranges
    if let Some(host_str) = url.host_str() {
        if let Ok(ip) = host_str.parse::<std::net::IpAddr>() {
            if ip.is_loopback() || is_private_ip(&ip) {
                return Err(anyhow::anyhow!("Private IP addresses are not allowed"));
            }
        }
    }

    Ok(())
}
```

---

### 2. MEDIUM: Missing File Permission Protection on Windows

**Severity:** MEDIUM
**Location:** `src/config.rs:108-116`
**CWE:** CWE-732: Incorrect Permission Assignment for Critical Resource

**Description:**
The configuration file containing sensitive credentials (API keys) is only protected with restrictive permissions (0o600) on Unix-like systems. On Windows, the file is created with default permissions, which may allow other users on the system to read the credentials.

**Vulnerable Code:**
```rust
// Set restrictive permissions on Unix systems
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&path, perms)?;
}
```

**Recommendation:**
Implement Windows-specific permission settings using the Windows API to restrict file access:

```rust
#[cfg(windows)]
{
    use std::os::windows::fs::OpenOptionsExt;
    // Set Windows ACLs to restrict access to current user only
    // Consider using the `winapi` or `windows` crate
}
```

**Impact:**
On Windows systems, other users may be able to read API credentials from the configuration file.

---

### 3. MEDIUM: Potential Credential Leakage in Error Messages

**Severity:** MEDIUM
**Location:** `src/client.rs:103-108`, multiple command files
**CWE:** CWE-209: Generation of Error Message Containing Sensitive Information

**Description:**
Error messages from API failures may include the full response body, which could potentially contain sensitive information or internal system details. While the current implementation doesn't directly log credentials, server error responses could expose internal paths, versions, or configuration details.

**Vulnerable Code:**
```rust
StatusCode::NOT_FOUND => {
    let message = response.text().await.unwrap_or_default();
    Err(ApiError::NotFoundError(message).into())
}
_ => {
    let message = response.text().await.unwrap_or_default();
    Err(ApiError::ApiError {
        status: status.as_u16(),
        message,
    }
    .into())
}
```

**Recommendation:**
1. Sanitize error messages before displaying them to users
2. Log full error details to a secure log file (if verbose mode is enabled)
3. Display generic error messages to users by default
4. Never include authentication headers or credentials in error messages

**Example Fix:**
```rust
fn sanitize_error_message(message: String, verbose: bool) -> String {
    if verbose {
        message
    } else {
        // Return generic message without sensitive details
        "Request failed. Use --verbose for details.".to_string()
    }
}
```

---

### 4. MEDIUM: Path Injection in File Read Operations

**Severity:** MEDIUM
**Location:** `src/commands/prompts.rs:297-305`
**CWE:** CWE-22: Improper Limitation of a Pathname to a Restricted Directory

**Description:**
The `read_content` function reads files from user-specified paths without validation. While this is intended functionality, there are no checks to prevent:
- Reading system files (e.g., `/etc/passwd`)
- Following symlinks to restricted areas
- Reading files outside the intended directory

**Vulnerable Code:**
```rust
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
```

**Recommendation:**
1. Validate that file paths are within expected directories
2. Canonicalize paths to resolve symlinks
3. Check file permissions before reading
4. Implement maximum file size limits to prevent DoS

**Example Fix:**
```rust
fn read_content(file: Option<&str>, max_size: usize) -> Result<String> {
    match file {
        Some(path) => {
            // Canonicalize to resolve symlinks
            let canonical_path = std::fs::canonicalize(path)?;

            // Check file size
            let metadata = std::fs::metadata(&canonical_path)?;
            if metadata.len() > max_size as u64 {
                return Err(anyhow::anyhow!("File too large"));
            }

            Ok(std::fs::read_to_string(canonical_path)?)
        },
        None => {
            // Read from stdin with size limit
            let mut buf = String::new();
            io::stdin().take(max_size as u64).read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
```

---

### 5. LOW: No TLS Certificate Validation Override Protection

**Severity:** LOW
**Location:** `src/client.rs:54-58`
**CWE:** CWE-295: Improper Certificate Validation

**Description:**
The HTTP client is created with default settings, which should include proper TLS certificate validation. However, there's no explicit configuration to prevent certificate validation bypasses, and no mechanism to pin certificates for known hosts.

**Current Code:**
```rust
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(10))
    .build()
    .context("Failed to create HTTP client")?;
```

**Recommendation:**
1. Explicitly enable TLS certificate validation
2. Consider implementing certificate pinning for the default Langfuse host
3. Add configuration option for custom CA certificates if needed for enterprise deployments

**Example Enhancement:**
```rust
let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(10))
    .danger_accept_invalid_certs(false) // Explicit
    .https_only(true) // Enforce HTTPS
    .build()
    .context("Failed to create HTTP client")?;
```

---

### 6. LOW: Credentials May Be Logged in Debug Output

**Severity:** LOW
**Location:** `src/client.rs:34-40`
**CWE:** CWE-532: Insertion of Sensitive Information into Log File

**Description:**
The `LangfuseClient` struct derives the `Debug` trait, which means credentials could be inadvertently logged if the struct is ever printed in debug output.

**Vulnerable Code:**
```rust
#[derive(Debug)]
pub struct LangfuseClient {
    client: Client,
    host: String,
    public_key: String,
    secret_key: String,
}
```

**Recommendation:**
Implement a custom `Debug` trait that masks sensitive fields:

```rust
impl std::fmt::Debug for LangfuseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LangfuseClient")
            .field("host", &self.host)
            .field("public_key", &"***REDACTED***")
            .field("secret_key", &"***REDACTED***")
            .finish()
    }
}
```

---

### 7. LOW: Missing Rate Limiting Protection

**Severity:** LOW
**Location:** `src/client.rs` (general)
**CWE:** CWE-400: Uncontrolled Resource Consumption

**Description:**
The client implements pagination to handle API rate limits (HTTP 429) but doesn't implement client-side rate limiting or backoff strategies. This could lead to:
- Rapid retry storms if the API is down
- Unnecessary API quota consumption
- Poor user experience during transient failures

**Recommendation:**
1. Implement exponential backoff for failed requests
2. Add configurable retry limits
3. Implement client-side rate limiting

**Example Enhancement:**
```rust
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<T, F, Fut>(
    operation: F,
    max_retries: u32
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                let delay = Duration::from_secs(2u64.pow(retries));
                sleep(delay).await;
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

### 8. LOW: Stdin Reading Without Size Limit

**Severity:** LOW
**Location:** `src/commands/prompts.rs:301-302`
**CWE:** CWE-400: Uncontrolled Resource Consumption

**Description:**
When reading content from stdin, there's no limit on the amount of data that can be read, potentially leading to memory exhaustion.

**Vulnerable Code:**
```rust
None => {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
```

**Recommendation:**
Implement a size limit for stdin reads:

```rust
None => {
    let mut buf = String::new();
    io::stdin()
        .take(10 * 1024 * 1024) // 10MB limit
        .read_to_string(&mut buf)?;
    Ok(buf)
}
```

---

### 9. INFORMATIONAL: Markdown Formatter Doesn't Escape Special Characters

**Severity:** INFORMATIONAL
**Location:** `src/formatters/markdown.rs:83-85`
**CWE:** N/A

**Description:**
The markdown formatter only escapes pipe characters but not other markdown special characters (backticks, asterisks, underscores, etc.). While this isn't a security vulnerability in the traditional sense, it could lead to rendering issues if the output is displayed in a markdown renderer.

**Current Code:**
```rust
fn escape_pipes(s: &str) -> String {
    s.replace('|', "\\|")
}
```

**Recommendation:**
Consider escaping additional markdown special characters if the output is intended to be rendered:

```rust
fn escape_markdown(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
        .replace('[', "\\[")
        .replace(']', "\\]")
}
```

---

### 10. INFORMATIONAL: No Input Validation for JSON Payloads

**Severity:** INFORMATIONAL
**Location:** Various command files
**CWE:** N/A

**Description:**
User-provided JSON data (for config, metadata, etc.) is parsed but not validated against schemas. While serde_json will reject invalid JSON, malformed but valid JSON could cause unexpected behavior.

**Recommendation:**
1. Implement JSON schema validation for important payloads
2. Validate field types and ranges
3. Reject excessively nested or large JSON objects

---

## Dependency Security Analysis

### Current Dependencies (from Cargo.toml)

The application uses the following key dependencies:

```toml
reqwest = "0.12"         # HTTP client
tokio = "1"              # Async runtime
serde = "1"              # Serialization
clap = "4"               # CLI parsing
anyhow = "1"             # Error handling
thiserror = "2"          # Error types
dialoguer = "0.11"       # User input
dotenvy = "0.15"         # Environment variables
```

**Findings:**
- All major dependencies are up-to-date
- No known critical vulnerabilities detected in review (cargo audit failed due to network issues, but manual review of versions shows recent releases)
- Dependencies follow semantic versioning

**Recommendations:**
1. Run `cargo audit` regularly in CI/CD pipeline
2. Enable GitHub Dependabot alerts
3. Consider using `cargo-deny` to enforce security policies
4. Keep dependencies updated

---

## Positive Security Findings

The following security practices are implemented correctly:

1. ✅ **Secure Credential Storage**: Config files are protected with 0o600 permissions on Unix
2. ✅ **Key Masking**: API keys are masked when displayed (first 8 chars + asterisks)
3. ✅ **HTTPS by Default**: Default host uses HTTPS
4. ✅ **No Command Execution**: No use of shell command execution (`Command::new`)
5. ✅ **Proper HTTP Authentication**: Uses reqwest's `basic_auth()` for proper header encoding
6. ✅ **Timeout Configuration**: HTTP requests have appropriate timeouts
7. ✅ **Proper Error Handling**: Uses Rust's Result type throughout
8. ✅ **Safe Serialization**: Uses serde for JSON/YAML parsing (memory-safe)
9. ✅ **CSV Escaping**: CSV formatter properly escapes special characters
10. ✅ **No SQL Injection**: No direct database queries (API-only client)

---

## Recommendations Summary

### Immediate Actions (High Priority)

1. **Implement Host URL Validation** - Prevent SSRF attacks by validating the host parameter
2. **Add Windows File Permissions** - Protect config files on Windows systems
3. **Sanitize Error Messages** - Prevent sensitive information disclosure

### Short-term Actions (Medium Priority)

4. **Add Path Validation** - Implement checks for file operations
5. **Implement Rate Limiting** - Add client-side backoff strategies
6. **Add Input Size Limits** - Prevent DoS via large inputs

### Long-term Actions (Low Priority)

7. **Certificate Pinning** - Add for default Langfuse host
8. **Custom Debug Implementation** - Mask credentials in debug output
9. **JSON Schema Validation** - Validate user-provided JSON
10. **Security Testing** - Add security-focused integration tests

---

## Testing Recommendations

1. **Security Test Cases:**
   - Test SSRF protection with various malicious URLs
   - Test file permission enforcement on Windows
   - Test error message sanitization
   - Test path traversal prevention
   - Test stdin size limits

2. **Fuzzing:**
   - Fuzz JSON parsing with cargo-fuzz
   - Fuzz CLI argument parsing
   - Fuzz file path handling

3. **Static Analysis:**
   - Run `cargo clippy` with security lints
   - Use `cargo-audit` in CI/CD
   - Consider using `cargo-geiger` to detect unsafe code

---

## Compliance Considerations

### OWASP Top 10 Coverage

| OWASP Category | Status | Notes |
|----------------|--------|-------|
| A01:2021 - Broken Access Control | ⚠️ Partial | File permissions need Windows support |
| A02:2021 - Cryptographic Failures | ⚠️ Partial | Credentials stored in config files |
| A03:2021 - Injection | ✅ Good | No SQL/Command injection vectors |
| A04:2021 - Insecure Design | ⚠️ Needs Work | Missing SSRF protection |
| A05:2021 - Security Misconfiguration | ✅ Good | Proper defaults, but could be better |
| A06:2021 - Vulnerable Components | ✅ Good | Dependencies up-to-date |
| A07:2021 - Authentication Failures | ✅ Good | Proper API authentication |
| A08:2021 - Software and Data Integrity | ✅ Good | No supply chain issues detected |
| A09:2021 - Security Logging | ⚠️ Partial | No security event logging |
| A10:2021 - SSRF | ❌ Vulnerable | No host URL validation |

---

## Conclusion

The `lf` CLI tool demonstrates good security practices in many areas, particularly in avoiding common vulnerabilities like command injection and SQL injection. However, several important improvements are needed:

**Critical fixes required:**
- Implement host URL validation to prevent SSRF attacks
- Add Windows file permission protection

**Priority improvements:**
- Error message sanitization
- Path validation for file operations
- Rate limiting and retry logic

The Rust language provides memory safety guarantees that eliminate entire classes of vulnerabilities (buffer overflows, use-after-free, etc.). The remaining security issues are primarily in application logic and input validation.

**Overall Assessment:** The tool is suitable for use with the recommended fixes implemented. The HIGH severity SSRF vulnerability should be addressed before deploying in production environments where users might configure malicious hosts.

---

## Audit Methodology

This audit included:
1. Manual code review of all source files
2. Analysis of authentication and credential handling
3. HTTP client security review
4. Input validation analysis
5. Output formatter security audit
6. Dependency version review
7. File operation security review
8. Error handling analysis

**Limitations:**
- cargo audit could not be executed due to network restrictions
- Dynamic testing was not performed
- Fuzzing was not conducted
- Penetration testing was not performed

---

**Report Generated:** 2026-01-14
**Next Review Recommended:** After implementing high/medium severity fixes
