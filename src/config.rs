use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::types::OutputFormat;

const DEFAULT_HOST: &str = "https://cloud.langfuse.com";
const DEFAULT_PROFILE: &str = "default";
const DEFAULT_LIMIT: u32 = 50;

/// Profile configuration stored in config file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub public_key: Option<String>,
    pub secret_key: Option<String>,
    pub host: Option<String>,
}

/// Configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

/// Runtime configuration with resolved values
#[derive(Debug, Clone)]
pub struct Config {
    pub public_key: Option<String>,
    pub secret_key: Option<String>,
    pub host: String,
    pub profile: String,
    pub format: OutputFormat,
    pub limit: u32,
    pub page: u32,
    pub output: Option<String>,
    pub verbose: bool,
    pub no_color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            public_key: None,
            secret_key: None,
            host: DEFAULT_HOST.to_string(),
            profile: DEFAULT_PROFILE.to_string(),
            format: OutputFormat::Table,
            limit: DEFAULT_LIMIT,
            page: 1,
            output: None,
            verbose: false,
            no_color: false,
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "langfuse") {
            let config_dir = proj_dirs.config_dir();
            Some(config_dir.join("config.yml"))
        } else {
            // Fallback to ~/.langfuse/config.yml
            dirs::home_dir().map(|home| home.join(".langfuse").join("config.yml"))
        }
    }

    /// Load configuration file
    pub fn load_config_file() -> Result<ConfigFile> {
        let path = Self::config_path();

        if let Some(path) = path {
            if path.exists() {
                let contents = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file: {:?}", path))?;
                let config: ConfigFile = serde_yaml::from_str(&contents)
                    .with_context(|| "Failed to parse config file")?;
                return Ok(config);
            }
        }

        Ok(ConfigFile::default())
    }

    /// Save configuration file
    pub fn save_config_file(config_file: &ConfigFile) -> Result<()> {
        let path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config file path"))?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let contents = serde_yaml::to_string(config_file)
            .with_context(|| "Failed to serialize config")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Load configuration with priority: CLI options > env vars > config file > defaults
    pub fn load(
        profile: Option<&str>,
        public_key: Option<&str>,
        secret_key: Option<&str>,
        host: Option<&str>,
        format: Option<OutputFormat>,
        limit: Option<u32>,
        page: Option<u32>,
        output: Option<&str>,
        verbose: bool,
        no_color: bool,
    ) -> Result<Self> {
        let profile_name = profile
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LANGFUSE_PROFILE").ok())
            .unwrap_or_else(|| DEFAULT_PROFILE.to_string());

        // Load config file
        let config_file = Self::load_config_file().unwrap_or_default();
        let file_profile = config_file.profiles.get(&profile_name);

        // Resolve public key: CLI > env > config file
        let resolved_public_key = public_key
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LANGFUSE_PUBLIC_KEY").ok())
            .or_else(|| file_profile.and_then(|p| p.public_key.clone()));

        // Resolve secret key: CLI > env > config file
        let resolved_secret_key = secret_key
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LANGFUSE_SECRET_KEY").ok())
            .or_else(|| file_profile.and_then(|p| p.secret_key.clone()));

        // Resolve host: CLI > env > config file > default
        let resolved_host = host
            .map(|s| s.to_string())
            .or_else(|| std::env::var("LANGFUSE_HOST").ok())
            .or_else(|| file_profile.and_then(|p| p.host.clone()))
            .unwrap_or_else(|| DEFAULT_HOST.to_string());

        Ok(Self {
            public_key: resolved_public_key,
            secret_key: resolved_secret_key,
            host: resolved_host,
            profile: profile_name,
            format: format.unwrap_or(OutputFormat::Table),
            limit: limit.unwrap_or(DEFAULT_LIMIT),
            page: page.unwrap_or(1),
            output: output.map(|s| s.to_string()),
            verbose,
            no_color,
        })
    }

    /// Check if configuration has required credentials
    pub fn is_valid(&self) -> bool {
        self.public_key.is_some() && self.secret_key.is_some() && !self.host.is_empty()
    }

    /// Set a profile in the config file
    pub fn set_profile(
        profile_name: &str,
        public_key: &str,
        secret_key: &str,
        host: Option<&str>,
    ) -> Result<()> {
        let mut config_file = Self::load_config_file().unwrap_or_default();

        config_file.profiles.insert(
            profile_name.to_string(),
            Profile {
                public_key: Some(public_key.to_string()),
                secret_key: Some(secret_key.to_string()),
                host: host.map(|s| s.to_string()),
            },
        );

        Self::save_config_file(&config_file)
    }

    /// Get a profile from the config file
    pub fn get_profile(profile_name: &str) -> Result<Option<Profile>> {
        let config_file = Self::load_config_file()?;
        Ok(config_file.profiles.get(profile_name).cloned())
    }

    /// List all profiles
    pub fn list_profiles() -> Result<Vec<String>> {
        let config_file = Self::load_config_file()?;
        Ok(config_file.profiles.keys().cloned().collect())
    }

    /// Mask a key for display (show first 8 chars + asterisks)
    pub fn mask_key(key: &str) -> String {
        if key.len() <= 8 {
            "*".repeat(key.len())
        } else {
            format!("{}********", &key[..8])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    // ========== Config Default Tests ==========

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(config.public_key.is_none());
        assert!(config.secret_key.is_none());
        assert_eq!(config.host, "https://cloud.langfuse.com");
        assert_eq!(config.profile, "default");
        assert_eq!(config.format, OutputFormat::Table);
        assert_eq!(config.limit, 50);
        assert_eq!(config.page, 1);
        assert!(config.output.is_none());
        assert!(!config.verbose);
        assert!(!config.no_color);
    }

    // ========== Config Validation Tests ==========

    #[test]
    fn test_config_is_valid_with_all_credentials() {
        let config = Config {
            public_key: Some("pk-test-123".to_string()),
            secret_key: Some("sk-test-456".to_string()),
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        assert!(config.is_valid());
    }

    #[test]
    fn test_config_is_invalid_without_public_key() {
        let config = Config {
            public_key: None,
            secret_key: Some("sk-test-456".to_string()),
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        assert!(!config.is_valid());
    }

    #[test]
    fn test_config_is_invalid_without_secret_key() {
        let config = Config {
            public_key: Some("pk-test-123".to_string()),
            secret_key: None,
            host: "https://example.com".to_string(),
            ..Default::default()
        };

        assert!(!config.is_valid());
    }

    #[test]
    fn test_config_is_invalid_with_empty_host() {
        let config = Config {
            public_key: Some("pk-test-123".to_string()),
            secret_key: Some("sk-test-456".to_string()),
            host: "".to_string(),
            ..Default::default()
        };

        assert!(!config.is_valid());
    }

    // ========== Key Masking Tests ==========

    #[test]
    fn test_mask_key_short_key() {
        assert_eq!(Config::mask_key("abc"), "***");
        assert_eq!(Config::mask_key("12345678"), "********");
    }

    #[test]
    fn test_mask_key_long_key() {
        assert_eq!(Config::mask_key("pk-test-1234567890"), "pk-test-********");
        assert_eq!(
            Config::mask_key("sk-lf-1234567890abcdef"),
            "sk-lf-12********"
        );
    }

    #[test]
    fn test_mask_key_empty() {
        assert_eq!(Config::mask_key(""), "");
    }

    #[test]
    fn test_mask_key_exactly_8_chars() {
        assert_eq!(Config::mask_key("abcdefgh"), "********");
    }

    #[test]
    fn test_mask_key_9_chars() {
        assert_eq!(Config::mask_key("abcdefghi"), "abcdefgh********");
    }

    // ========== Profile Tests ==========

    #[test]
    fn test_profile_default() {
        let profile = Profile::default();

        assert!(profile.public_key.is_none());
        assert!(profile.secret_key.is_none());
        assert!(profile.host.is_none());
    }

    #[test]
    fn test_profile_serialize() {
        let profile = Profile {
            public_key: Some("pk-123".to_string()),
            secret_key: Some("sk-456".to_string()),
            host: Some("https://custom.com".to_string()),
        };

        let yaml = serde_yaml::to_string(&profile).unwrap();

        assert!(yaml.contains("public_key: pk-123"));
        assert!(yaml.contains("secret_key: sk-456"));
        assert!(yaml.contains("host: https://custom.com"));
    }

    #[test]
    fn test_profile_deserialize() {
        let yaml = r#"
public_key: pk-test
secret_key: sk-test
host: https://test.langfuse.com
"#;

        let profile: Profile = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(profile.public_key, Some("pk-test".to_string()));
        assert_eq!(profile.secret_key, Some("sk-test".to_string()));
        assert_eq!(profile.host, Some("https://test.langfuse.com".to_string()));
    }

    // ========== ConfigFile Tests ==========

    #[test]
    fn test_config_file_default() {
        let config_file = ConfigFile::default();

        assert!(config_file.profiles.is_empty());
    }

    #[test]
    fn test_config_file_serialize() {
        let mut config_file = ConfigFile::default();
        config_file.profiles.insert(
            "default".to_string(),
            Profile {
                public_key: Some("pk-default".to_string()),
                secret_key: Some("sk-default".to_string()),
                host: None,
            },
        );
        config_file.profiles.insert(
            "production".to_string(),
            Profile {
                public_key: Some("pk-prod".to_string()),
                secret_key: Some("sk-prod".to_string()),
                host: Some("https://prod.langfuse.com".to_string()),
            },
        );

        let yaml = serde_yaml::to_string(&config_file).unwrap();

        assert!(yaml.contains("default:"));
        assert!(yaml.contains("production:"));
        assert!(yaml.contains("pk-default"));
        assert!(yaml.contains("pk-prod"));
    }

    #[test]
    fn test_config_file_deserialize() {
        let yaml = r#"
profiles:
  default:
    public_key: pk-test
    secret_key: sk-test
  staging:
    public_key: pk-staging
    secret_key: sk-staging
    host: https://staging.langfuse.com
"#;

        let config_file: ConfigFile = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config_file.profiles.len(), 2);
        assert!(config_file.profiles.contains_key("default"));
        assert!(config_file.profiles.contains_key("staging"));

        let staging = config_file.profiles.get("staging").unwrap();
        assert_eq!(staging.public_key, Some("pk-staging".to_string()));
        assert_eq!(
            staging.host,
            Some("https://staging.langfuse.com".to_string())
        );
    }

    // ========== Config Load Tests ==========

    #[test]
    fn test_config_load_with_cli_options() {
        // CLI options should override everything
        let config = Config::load(
            Some("custom-profile"),
            Some("pk-cli"),
            Some("sk-cli"),
            Some("https://cli.example.com"),
            Some(OutputFormat::Json),
            Some(100),
            Some(5),
            Some("/tmp/output.json"),
            true,
            true,
        )
        .unwrap();

        assert_eq!(config.public_key, Some("pk-cli".to_string()));
        assert_eq!(config.secret_key, Some("sk-cli".to_string()));
        assert_eq!(config.host, "https://cli.example.com");
        assert_eq!(config.profile, "custom-profile");
        assert_eq!(config.format, OutputFormat::Json);
        assert_eq!(config.limit, 100);
        assert_eq!(config.page, 5);
        assert_eq!(config.output, Some("/tmp/output.json".to_string()));
        assert!(config.verbose);
        assert!(config.no_color);
    }

    #[test]
    fn test_config_load_with_defaults() {
        // Clear environment variables that might interfere
        env::remove_var("LANGFUSE_PUBLIC_KEY");
        env::remove_var("LANGFUSE_SECRET_KEY");
        env::remove_var("LANGFUSE_HOST");
        env::remove_var("LANGFUSE_PROFILE");

        let config = Config::load(
            None, None, None, None, None, None, None, None, false, false,
        )
        .unwrap();

        assert!(config.public_key.is_none());
        assert!(config.secret_key.is_none());
        assert_eq!(config.host, "https://cloud.langfuse.com");
        assert_eq!(config.profile, "default");
        assert_eq!(config.format, OutputFormat::Table);
        assert_eq!(config.limit, 50);
        assert_eq!(config.page, 1);
    }

    #[test]
    fn test_config_load_format_options() {
        let config_table = Config::load(
            None, None, None, None,
            Some(OutputFormat::Table),
            None, None, None, false, false,
        )
        .unwrap();
        assert_eq!(config_table.format, OutputFormat::Table);

        let config_json = Config::load(
            None, None, None, None,
            Some(OutputFormat::Json),
            None, None, None, false, false,
        )
        .unwrap();
        assert_eq!(config_json.format, OutputFormat::Json);

        let config_csv = Config::load(
            None, None, None, None,
            Some(OutputFormat::Csv),
            None, None, None, false, false,
        )
        .unwrap();
        assert_eq!(config_csv.format, OutputFormat::Csv);

        let config_md = Config::load(
            None, None, None, None,
            Some(OutputFormat::Markdown),
            None, None, None, false, false,
        )
        .unwrap();
        assert_eq!(config_md.format, OutputFormat::Markdown);
    }

    // ========== Config File Save/Load Tests ==========

    #[test]
    fn test_save_and_load_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        // Create a config file
        let mut config_file = ConfigFile::default();
        config_file.profiles.insert(
            "test-profile".to_string(),
            Profile {
                public_key: Some("pk-save-test".to_string()),
                secret_key: Some("sk-save-test".to_string()),
                host: Some("https://save-test.com".to_string()),
            },
        );

        // Write it manually to the temp location
        let parent = config_path.parent().unwrap();
        fs::create_dir_all(parent).unwrap();
        let contents = serde_yaml::to_string(&config_file).unwrap();
        fs::write(&config_path, contents).unwrap();

        // Read it back
        let read_contents = fs::read_to_string(&config_path).unwrap();
        let loaded: ConfigFile = serde_yaml::from_str(&read_contents).unwrap();

        assert_eq!(loaded.profiles.len(), 1);
        let profile = loaded.profiles.get("test-profile").unwrap();
        assert_eq!(profile.public_key, Some("pk-save-test".to_string()));
        assert_eq!(profile.secret_key, Some("sk-save-test".to_string()));
        assert_eq!(profile.host, Some("https://save-test.com".to_string()));
    }

    // ========== Config Path Tests ==========

    #[test]
    fn test_config_path_returns_some() {
        // This should always return Some on systems with home directories
        let path = Config::config_path();
        // On most systems this will be Some, but we handle both cases
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("langfuse"));
            assert!(p.to_string_lossy().ends_with("config.yml"));
        }
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_config_with_unicode_values() {
        let config = Config::load(
            Some("профиль"),
            Some("pk-测试"),
            Some("sk-テスト"),
            Some("https://example.com/путь"),
            None,
            None,
            None,
            Some("/tmp/出力.json"),
            false,
            false,
        )
        .unwrap();

        assert_eq!(config.profile, "профиль");
        assert_eq!(config.public_key, Some("pk-测试".to_string()));
        assert_eq!(config.secret_key, Some("sk-テスト".to_string()));
        assert_eq!(config.output, Some("/tmp/出力.json".to_string()));
    }

    #[test]
    fn test_mask_key_unicode() {
        // Unicode characters should be handled properly
        assert_eq!(Config::mask_key("🔑🔑🔑"), "***");
        // Note: Unicode characters may have multi-byte representations
        let key = "pk-日本語テスト";
        let masked = Config::mask_key(key);
        // The masking is byte-based, so we check it doesn't panic
        assert!(!masked.is_empty());
    }

    #[test]
    fn test_config_file_empty_profiles() {
        let yaml = "profiles: {}";
        let config_file: ConfigFile = serde_yaml::from_str(yaml).unwrap();
        assert!(config_file.profiles.is_empty());
    }

    #[test]
    fn test_config_file_missing_profiles_key() {
        let yaml = "other_key: value";
        let config_file: ConfigFile = serde_yaml::from_str(yaml).unwrap();
        assert!(config_file.profiles.is_empty());
    }

    #[test]
    fn test_profile_partial_fields() {
        let yaml = r#"
public_key: only-public-key
"#;

        let profile: Profile = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(profile.public_key, Some("only-public-key".to_string()));
        assert!(profile.secret_key.is_none());
        assert!(profile.host.is_none());
    }
}
