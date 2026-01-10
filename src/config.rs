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
