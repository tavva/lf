# Security Audit Report

**Project:** lf (Langfuse CLI)
**Date:** 2026-01-10
**Auditor:** Claude (Automated Security Audit)

## Executive Summary

This security audit reviewed the `lf` Rust CLI for the Langfuse LLM observability platform. The codebase demonstrates generally good security practices, leveraging Rust's memory safety guarantees. However, several issues were identified ranging from a deprecated dependency to potential path traversal concerns.

**Risk Summary:**
- Critical: 1 (deprecated dependency)
- High: 1 (URL path injection)
- Medium: 3 (credential exposure, file permission race, arbitrary file write)
- Low: 3 (host validation, UTF-8 truncation, error message leakage)

---

## Critical Severity

### 1. Deprecated serde_yaml Dependency

**Location:** `Cargo.toml:24`
**Issue:** The `serde_yaml` crate (v0.9.34) is officially deprecated and no longer maintained.

```toml
serde_yaml = "0.9"
```

**Risk:** Deprecated dependencies may contain unpatched security vulnerabilities and will not receive future security updates.

**Recommendation:** Migrate to a maintained alternative such as:
- `serde_yml` (community fork)
- `yaml-rust2` with serde support
- Consider JSON for config if YAML features aren't required

---

## High Severity

### 2. URL Path Injection in API Requests

**Locations:**
- `src/client.rs:239` - `get_trace()`
- `src/client.rs:297` - `get_session()`
- `src/client.rs:371` - `get_observation()`
- `src/client.rs:433` - `get_score()`

**Issue:** Resource IDs are directly interpolated into URL paths without validation or encoding:

```rust
pub async fn get_trace(&self, id: &str) -> Result<Trace> {
    self.get(&format!("/traces/{}", id), &[]).await
}
```

**Attack Scenario:** If an attacker controls the ID parameter (e.g., via script input), they could inject path segments:
- ID: `../../../admin/config` could traverse API paths
- ID: `valid-id?admin=true` could inject query parameters

**Recommendation:**
1. URL-encode the ID before interpolation using `percent_encoding` crate
2. Validate ID format (alphanumeric + common ID characters only)
3. Example fix:
```rust
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

let encoded_id = utf8_percent_encode(id, NON_ALPHANUMERIC).to_string();
self.get(&format!("/traces/{}", encoded_id), &[]).await
```

---

## Medium Severity

### 3. Credentials Exposed via CLI Arguments

**Location:** All command modules (e.g., `src/commands/traces.rs:57-62`)

**Issue:** Sensitive credentials can be passed via command-line arguments:

```rust
#[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
public_key: Option<String>,

#[arg(long, env = "LANGFUSE_SECRET_KEY")]
secret_key: Option<String>,
```

**Risk:** CLI arguments are visible in:
- Process listings (`ps aux`, `/proc/*/cmdline`)
- Shell history files
- System logs and audit trails
- Parent process memory

**Recommendation:**
1. Add a warning when credentials are passed via CLI
2. Document that environment variables or config files are preferred
3. Consider removing CLI args for credentials entirely, requiring env vars or config file

---

### 4. Config File Permission Race Condition

**Location:** `src/config.rs:96-114`

**Issue:** The config file is written first, then permissions are restricted:

```rust
fs::write(&path, contents)?;

#[cfg(unix)]
{
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&path, perms)?;
}
```

**Risk:** There's a brief window where the file exists with default permissions (typically 0644), potentially exposing credentials to other users on multi-user systems.

**Recommendation:** Use atomic file creation with pre-set permissions:

```rust
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

#[cfg(unix)]
let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(0o600)
    .open(&path)?;
```

Or use `tempfile` crate to create in temp location then rename atomically.

---

### 5. Arbitrary File Write via --output Flag

**Location:** `src/commands/mod.rs:17`

**Issue:** The output file path is used directly without validation:

```rust
if let Some(path) = output_path {
    fs::write(path, content)?;
```

**Risk:** Could overwrite sensitive files if a user is tricked into using a malicious path or if the path comes from an untrusted source:
- Overwriting `.bashrc`, `.ssh/authorized_keys`
- Writing to system directories if running as root

**Recommendation:**
1. Validate the path doesn't escape intended directories
2. Warn before overwriting existing files
3. Block paths to sensitive locations (dotfiles, system directories)

---

## Low Severity

### 6. No Host URL Validation

**Locations:**
- `src/config.rs:154-158`
- `src/client.rs:69-70`

**Issue:** The host URL is accepted without validation:

```rust
let url = format!("{}/api/public{}", self.host, path);
```

**Risk:**
- HTTP connections could be established if user specifies `http://`
- Malformed URLs could cause unexpected behavior
- Potential for SSRF if host is controlled by untrusted input

**Recommendation:**
1. Validate URL format
2. Require HTTPS scheme
3. Validate against allowlist of known Langfuse hosts or warn for custom hosts

---

### 7. UTF-8 Truncation Panic Risk

**Location:** `src/formatters/table.rs:89-95`

**Issue:** String truncation uses byte indexing which can panic on multi-byte UTF-8 characters:

```rust
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])  // PANIC if max_len splits a UTF-8 char
    }
}
```

**Risk:** Panics when processing data containing emoji, CJK characters, or other multi-byte Unicode.

**Recommendation:** Use character-aware truncation:

```rust
fn truncate_string(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_chars).collect::<String>())
    }
}
```

---

### 8. Error Messages May Leak Sensitive Information

**Locations:**
- `src/client.rs:99-101`
- `src/client.rs:155-161`

**Issue:** Raw API error responses are included in error messages:

```rust
let message = response.text().await.unwrap_or_default();
Err(ApiError::NotFoundError(message).into())
```

**Risk:** API error responses might contain:
- Internal server details
- Database error messages
- Stack traces
- Other sensitive debugging information

**Recommendation:**
1. Truncate error messages to reasonable length
2. Strip or sanitize potential sensitive patterns
3. Log full errors only in verbose mode

---

## Positive Security Practices

The codebase demonstrates several good security practices:

| Practice | Location | Notes |
|----------|----------|-------|
| File permissions | `config.rs:108-113` | Sets 0o600 on Unix (with race condition noted above) |
| HTTPS default | `config.rs:10` | Default host is `https://cloud.langfuse.com` |
| Password masking | `commands/config.rs:81-83` | Uses `dialoguer::Password` for interactive input |
| Key display masking | `config.rs:213-221` | `mask_key()` hides most of credential in display |
| HTTP timeouts | `client.rs:53-55` | 30s read, 10s connect timeout prevents hanging |
| No shell execution | All commands | No user input passed to shell commands |
| Type safety | Throughout | Rust's type system prevents many vulnerabilities |
| Basic auth over HTTPS | `client.rs:71, 125` | Uses reqwest's `basic_auth` with HTTPS |

---

## Dependency Review

Current dependencies (from `Cargo.toml`):

| Dependency | Version | Status |
|------------|---------|--------|
| clap | 4 | ✅ Maintained |
| tokio | 1 | ✅ Maintained |
| reqwest | 0.12 | ✅ Maintained |
| serde | 1 | ✅ Maintained |
| serde_json | 1 | ✅ Maintained |
| **serde_yaml** | 0.9 | ⚠️ **DEPRECATED** |
| chrono | 0.4 | ✅ Maintained |
| tabled | 0.16 | ✅ Maintained (0.20 available) |
| csv | 1.3 | ✅ Maintained |
| directories | 5 | ✅ Maintained |
| dirs | 5 | ✅ Maintained |
| thiserror | 2 | ✅ Maintained |
| anyhow | 1 | ✅ Maintained |
| dialoguer | 0.11 | ✅ Maintained (0.12 available) |
| dotenvy | 0.15 | ✅ Maintained |

**Note:** `cargo audit` is not installed in the environment. It's recommended to run `cargo audit` regularly to check for known vulnerabilities.

---

## Recommendations Summary

### Immediate Actions (Critical/High)
1. Replace `serde_yaml` with maintained alternative
2. Add URL encoding for resource IDs in API paths

### Short-term Actions (Medium)
3. Fix config file permission race condition
4. Add warning for CLI credential usage
5. Validate output file paths

### Long-term Improvements (Low)
6. Add host URL validation
7. Fix UTF-8 truncation to use character boundaries
8. Sanitize error message content

---

## Testing Recommendations

1. **Install cargo-audit**: `cargo install cargo-audit && cargo audit`
2. **Fuzz testing**: Consider fuzzing the formatters and config parser
3. **Path injection tests**: Test with malicious IDs containing `../`, `?`, `#`, etc.
4. **Unicode stress tests**: Test with emoji, RTL text, and multi-byte characters

---

*This audit was performed through static code analysis. Runtime testing and penetration testing are recommended for production deployments.*
