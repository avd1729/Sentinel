use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for the Sentinel server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server listening configuration
    pub server: ServerConfig,
    
    /// Static file serving configuration
    pub static_files: StaticFilesConfig,
}

/// Server listening settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind to (e.g., "127.0.0.1:8080")
    pub listen_addr: String,
}

/// Configuration for serving static files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFilesConfig {
    /// Root directory for static files
    pub root: PathBuf,
    
    /// Default file to serve for directory requests
    pub index: String,
    
    /// Custom error pages
    #[serde(default)]
    pub error_pages: ErrorPages,
    
    /// Enable or disable directory listings (for future implementation)
    #[serde(default = "default_false")]
    pub directory_listing: bool,
}

/// Custom error page configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorPages {
    /// Custom 404 Not Found page (relative to static root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_found: Option<String>,
    
    /// Custom 400 Bad Request page (relative to static root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bad_request: Option<String>,
}

fn default_false() -> bool {
    false
}

impl Config {
    /// Load configuration from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Returns
    ///
    /// Returns the loaded configuration or an error if loading fails
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Load configuration with fallback to defaults
    ///
    /// Tries to load from `config.yaml`, falls back to environment variables,
    /// and finally to hardcoded defaults if neither is available.
    pub fn load() -> Self {
        // Try loading from config.yaml
        if let Ok(config) = Self::load_from_file("config.yaml") {
            tracing::info!("Loaded configuration from config.yaml");
            return config;
        }
        
        // Fallback to environment variables and defaults
        tracing::info!("Using default configuration");
        let listen_addr = std::env::var("LISTEN")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        
        Self {
            server: ServerConfig { listen_addr },
            static_files: StaticFilesConfig {
                root: PathBuf::from("public"),
                index: "index.html".to_string(),
                error_pages: ErrorPages::default(),
                directory_listing: false,
            },
        }
    }
}