use anyhow::{Context, Result};
use clap::Subcommand;
use dialoguer::{Input, Password};

use crate::client::LangfuseClient;
use crate::config::Config;

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Interactive configuration wizard
    Setup {
        /// Run in non-interactive mode using environment variables
        #[arg(long)]
        non_interactive: bool,
    },

    /// Set configuration for a profile
    Set {
        /// Profile name
        #[arg(short, long, default_value = "default")]
        profile: String,

        /// Langfuse public key
        #[arg(long, env = "LANGFUSE_PUBLIC_KEY")]
        public_key: String,

        /// Langfuse secret key
        #[arg(long, env = "LANGFUSE_SECRET_KEY")]
        secret_key: String,

        /// Langfuse host URL
        #[arg(long, env = "LANGFUSE_HOST")]
        host: Option<String>,
    },

    /// Show configuration for a profile
    Show {
        /// Profile name
        #[arg(short, long, default_value = "default")]
        profile: String,
    },

    /// List all configured profiles
    List,
}

impl ConfigCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            ConfigCommands::Setup { non_interactive } => {
                if *non_interactive {
                    self.setup_non_interactive().await
                } else {
                    self.setup_interactive().await
                }
            }
            ConfigCommands::Set {
                profile,
                public_key,
                secret_key,
                host,
            } => {
                self.set_config(profile, public_key, secret_key, host.as_deref())
                    .await
            }
            ConfigCommands::Show { profile } => self.show_config(profile),
            ConfigCommands::List => self.list_profiles(),
        }
    }

    async fn setup_interactive(&self) -> Result<()> {
        println!("Langfuse CLI Configuration Setup");
        println!("=================================\n");

        let profile: String = Input::new()
            .with_prompt("Profile name")
            .default("default".to_string())
            .interact_text()?;

        let public_key: String = Input::new().with_prompt("Public key").interact_text()?;

        let secret_key: String = Password::new().with_prompt("Secret key").interact()?;

        let host: String = Input::new()
            .with_prompt("Host URL")
            .default("https://cloud.langfuse.com".to_string())
            .interact_text()?;

        // Test connection
        println!("\nTesting connection...");
        let config = Config::load(
            Some(&profile),
            Some(&public_key),
            Some(&secret_key),
            Some(&host),
            None,
            None,
            None,
            None,
            false,
            false,
        )?;

        let client = LangfuseClient::new(&config)?;
        match client.test_connection().await {
            Ok(_) => {
                println!("Connection successful!");

                // Save configuration
                Config::set_profile(&profile, &public_key, &secret_key, Some(&host))?;
                println!("\nConfiguration saved to profile '{profile}'");

                if let Some(path) = Config::config_path() {
                    println!("Config file: {path:?}");
                }

                if profile != "default" {
                    println!("\nTo use this profile, either:");
                    println!("  lf traces list --profile {profile}");
                    println!("  export LANGFUSE_PROFILE={profile}");
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("Connection failed: {e}");
                Err(e)
            }
        }
    }

    async fn setup_non_interactive(&self) -> Result<()> {
        let profile = std::env::var("LANGFUSE_PROFILE").unwrap_or_else(|_| "default".to_string());
        let public_key =
            std::env::var("LANGFUSE_PUBLIC_KEY").context("LANGFUSE_PUBLIC_KEY not set")?;
        let secret_key =
            std::env::var("LANGFUSE_SECRET_KEY").context("LANGFUSE_SECRET_KEY not set")?;
        let host = std::env::var("LANGFUSE_HOST")
            .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string());

        // Test connection
        eprintln!("Testing connection...");
        let config = Config::load(
            Some(&profile),
            Some(&public_key),
            Some(&secret_key),
            Some(&host),
            None,
            None,
            None,
            None,
            false,
            false,
        )?;

        let client = LangfuseClient::new(&config)?;
        match client.test_connection().await {
            Ok(_) => {
                eprintln!("Connection successful!");

                // Save configuration
                Config::set_profile(&profile, &public_key, &secret_key, Some(&host))?;
                eprintln!("Configuration saved to profile '{profile}'");

                if profile != "default" {
                    eprintln!("\nTo use this profile, either:");
                    eprintln!("  lf traces list --profile {profile}");
                    eprintln!("  export LANGFUSE_PROFILE={profile}");
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("Connection failed: {e}");
                Err(e)
            }
        }
    }

    async fn set_config(
        &self,
        profile: &str,
        public_key: &str,
        secret_key: &str,
        host: Option<&str>,
    ) -> Result<()> {
        // Test connection before saving
        let test_config = Config::load(
            Some(profile),
            Some(public_key),
            Some(secret_key),
            host,
            None,
            None,
            None,
            None,
            false,
            false,
        )?;

        let client = LangfuseClient::new(&test_config)?;
        match client.test_connection().await {
            Ok(_) => {
                Config::set_profile(profile, public_key, secret_key, host)?;
                println!("Configuration saved to profile '{profile}'");
                if profile != "default" {
                    println!("\nTo use this profile, either:");
                    println!("  lf traces list --profile {profile}");
                    println!("  export LANGFUSE_PROFILE={profile}");
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Connection test failed: {e}");
                Err(e)
            }
        }
    }

    fn show_config(&self, profile_name: &str) -> Result<()> {
        match Config::get_profile(profile_name)? {
            Some(profile) => {
                println!("Profile: {profile_name}");
                println!("─────────────────────────────────");

                if let Some(pk) = &profile.public_key {
                    println!("Public Key: {}", Config::mask_key(pk));
                } else {
                    println!("Public Key: (not set)");
                }

                if let Some(sk) = &profile.secret_key {
                    println!("Secret Key: {}", Config::mask_key(sk));
                } else {
                    println!("Secret Key: (not set)");
                }

                if let Some(host) = &profile.host {
                    println!("Host: {host}");
                } else {
                    println!("Host: (default: https://cloud.langfuse.com)");
                }

                Ok(())
            }
            None => {
                eprintln!("Profile '{profile_name}' not found");
                std::process::exit(1);
            }
        }
    }

    fn list_profiles(&self) -> Result<()> {
        let profiles = Config::list_profiles()?;

        if profiles.is_empty() {
            println!("No profiles configured.");
            println!("Run 'lf config setup' to create a profile.");
        } else {
            println!("Configured profiles:");
            println!("─────────────────────");
            for profile in profiles {
                println!("  - {profile}");
            }
        }

        Ok(())
    }
}
