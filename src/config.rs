//! Configuration management for Code Graph Tool
//!
//! Supports loading configuration from environment variables and TOML files.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,

    /// LLM integration configuration
    pub llm: LlmConfig,

    /// Parser and query configuration
    pub parser: ParserConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: String,

    /// Maximum number of database connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
            max_connections: default_max_connections(),
        }
    }
}

fn default_db_path() -> String {
    "graphrag.db".to_string()
}

fn default_max_connections() -> u32 {
    5
}

/// LLM integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Base URL for LLM API
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,

    /// API key for authentication
    #[serde(default)]
    pub api_key: Option<String>,

    /// Model to use for analysis
    #[serde(default = "default_model")]
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: default_openai_base_url(),
            api_key: None,
            model: default_model(),
        }
    }
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

impl LlmConfig {
    /// Load API key from environment variable if not set in config
    pub fn api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
    }

    /// Get base URL, preferring environment variable
    pub fn base_url(&self) -> String {
        std::env::var("OPENAI_BASE_URL").unwrap_or(self.base_url.clone())
    }

    /// Get model, preferring environment variable
    pub fn model(&self) -> String {
        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| self.model.clone())
    }
}

/// Parser and query configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Maximum depth for dependency context retrieval
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,

    /// Default maximum tokens for context generation
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// List of supported file extensions
    #[serde(default = "default_supported_languages")]
    pub supported_languages: Vec<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_depth: default_max_depth(),
            max_tokens: default_max_tokens(),
            supported_languages: default_supported_languages(),
        }
    }
}

fn default_max_depth() -> u32 {
    2
}

fn default_max_tokens() -> usize {
    2000
}

fn default_supported_languages() -> Vec<String> {
    vec![
        "py".to_string(),
        "rs".to_string(),
        "js".to_string(),
        "ts".to_string(),
        "go".to_string(),
        "cpp".to_string(),
        "cc".to_string(),
        "cxx".to_string(),
        "c".to_string(),
        "h".to_string(),
    ]
}

impl Config {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> Self {
        info!("Loading configuration from environment");

        Self {
            database: DatabaseConfig {
                path: std::env::var("GRAPH_RAG_DB_PATH")
                    .unwrap_or_else(|_| "graphrag.db".to_string()),
                max_connections: std::env::var("GRAPH_RAG_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
            },
            llm: LlmConfig {
                base_url: std::env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                api_key: None, // Must be set via environment
                model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            },
            parser: ParserConfig {
                max_depth: std::env::var("GRAPH_RAG_MAX_DEPTH")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2),
                max_tokens: std::env::var("GRAPH_RAG_MAX_TOKENS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2000),
                supported_languages: default_supported_languages(),
            },
        }
    }

    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading configuration from file: {:?}", path.as_ref());

        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    /// Load configuration from file or fall back to environment variables
    pub fn from_file_or_env<P: AsRef<Path>>(path: P) -> Self {
        Self::from_file(path.as_ref()).unwrap_or_else(|_| {
            info!("Using default configuration (file not found or invalid)");
            Self::from_env()
        })
    }

    /// Get database URL
    pub fn db_url(&self) -> String {
        format!("sqlite://{}", self.database.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.database.path, "graphrag.db");
        assert_eq!(config.database.max_connections, 5);
        assert_eq!(config.llm.model, "gpt-4o-mini");
        assert_eq!(config.parser.max_depth, 2);
        assert_eq!(config.parser.max_tokens, 2000);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("OPENAI_BASE_URL", "https://custom.api.com/v1");
        std::env::set_var("OPENAI_MODEL", "gpt-4");
        std::env::set_var("GRAPH_RAG_MAX_DEPTH", "3");

        let config = Config::from_env();

        assert_eq!(config.llm.base_url, "https://custom.api.com/v1");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.parser.max_depth, 3);

        // Cleanup
        std::env::remove_var("OPENAI_BASE_URL");
        std::env::remove_var("OPENAI_MODEL");
        std::env::remove_var("GRAPH_RAG_MAX_DEPTH");
    }

    #[test]
    fn test_config_db_url() {
        let config = Config::default();
        assert_eq!(config.db_url(), "sqlite://graphrag.db");
    }
}
