//! HiveCode LLM Provider Abstraction Layer
//!
//! This crate provides a unified interface for interacting with multiple LLM providers
//! (OpenAI, Anthropic, Ollama, AWS Bedrock, Google Vertex AI, and any OpenAI-compatible endpoint).
//!
//! # Overview
//!
//! - **traits**: Core LlmProvider trait and related abstractions
//! - **types**: Common data structures for requests/responses
//! - **registry**: Provider management and selection
//! - **openai**: OpenAI and OpenAI-compatible endpoint provider
//! - **anthropic**: Anthropic Claude provider
//! - **ollama**: Local Ollama provider
//! - **bedrock**: AWS Bedrock provider for Claude models
//! - **vertex**: Google Vertex AI provider for Claude models
//! - **foundry**: Anthropic Foundry provider with custom base URL
//! - **error**: Error types and handling

pub mod anthropic;
pub mod bedrock;
pub mod error;
pub mod foundry;
pub mod ollama;
pub mod openai;
pub mod registry;
pub mod traits;
pub mod types;
pub mod vertex;

pub use anthropic::{AnthropicConfig, AnthropicProvider};
pub use bedrock::{BedrockConfig, BedrockProvider};
pub use error::ProviderError;
pub use foundry::{FoundryConfig, FoundryProvider};
pub use ollama::{OllamaConfig, OllamaProvider};
pub use openai::{OpenAiConfig, OpenAiProvider};
pub use registry::ProviderRegistry;
pub use traits::LlmProvider;
pub use types::{ChatMessage, ChatRequest, ChatResponse, ContentPart, ModelInfo, StreamEvent};
pub use vertex::{VertexConfig, VertexProvider};

pub type Result<T> = std::result::Result<T, ProviderError>;

use std::sync::Arc;
use tracing::{debug, info, warn};

/// Initialize providers from configuration
///
/// Creates a ProviderRegistry and populates it with available providers based on:
/// - Configuration file (api_key, base_url, default_model)
/// - Environment variables (ANTHROPIC_API_KEY, OPENAI_API_KEY, OLLAMA_BASE_URL)
/// - Health checks (Ollama availability at localhost:11434)
///
/// Provider preference order (for default):
/// 1. Anthropic (if API key available)
/// 2. OpenAI (if API key available)
/// 3. Ollama (if reachable)
///
/// # Arguments
///
/// * `config` - The HiveConfig containing provider configurations
///
/// # Returns
///
/// A populated ProviderRegistry with initialized providers
pub async fn initialize_providers(
    config: &hivecode_core::config::HiveConfig,
) -> Result<ProviderRegistry> {
    debug!("Initializing provider registry from configuration");

    let registry = ProviderRegistry::new();
    let mut default_provider: Option<String> = None;

    // Try to initialize Anthropic provider
    if let Some(provider_config) = config.providers.get("anthropic") {
        match initialize_anthropic_provider(provider_config) {
            Ok(provider) => {
                info!("Anthropic provider initialized successfully");
                registry.register("anthropic".to_string(), Arc::new(provider))?;

                // Set as default if no default is set yet
                if default_provider.is_none() {
                    default_provider = Some("anthropic".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to initialize Anthropic provider: {}", e);
            }
        }
    } else {
        // Try environment variable as fallback
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            match AnthropicProvider::builder()
                .api_key(&api_key)
                .build()
            {
                Ok(provider) => {
                    info!("Anthropic provider initialized from environment variable");
                    registry.register("anthropic".to_string(), Arc::new(provider))?;

                    if default_provider.is_none() {
                        default_provider = Some("anthropic".to_string());
                    }
                }
                Err(e) => {
                    warn!("Failed to build Anthropic provider from env var: {}", e);
                }
            }
        }
    }

    // Try to initialize OpenAI provider
    if let Some(provider_config) = config.providers.get("openai") {
        match initialize_openai_provider(provider_config) {
            Ok(provider) => {
                info!("OpenAI provider initialized successfully");
                registry.register("openai".to_string(), Arc::new(provider))?;

                // Set as default only if no default is set yet
                if default_provider.is_none() {
                    default_provider = Some("openai".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to initialize OpenAI provider: {}", e);
            }
        }
    } else {
        // Try environment variable as fallback
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let provider = OpenAiProvider::builder()
                .api_key(&api_key)
                .build();

            info!("OpenAI provider initialized from environment variable");
            registry.register("openai".to_string(), Arc::new(provider))?;

            if default_provider.is_none() {
                default_provider = Some("openai".to_string());
            }
        }
    }

    // Try to initialize Ollama provider (always attempt, as it's local)
    match initialize_ollama_provider(config).await {
        Ok(Some(provider)) => {
            info!("Ollama provider initialized successfully");
            registry.register("ollama".to_string(), Arc::new(provider))?;

            // Set as default only if no other provider is set
            if default_provider.is_none() {
                default_provider = Some("ollama".to_string());
            }
        }
        Ok(None) => {
            debug!("Ollama is not available (not running at localhost:11434)");
        }
        Err(e) => {
            debug!("Failed to initialize Ollama provider: {}", e);
        }
    }

    // Try to initialize AWS Bedrock provider
    if let Some(provider_config) = config.providers.get("bedrock") {
        match initialize_bedrock_provider(provider_config) {
            Ok(provider) => {
                info!("AWS Bedrock provider initialized successfully");
                registry.register("bedrock".to_string(), Arc::new(provider))?;

                // Set as default only if no default is set yet
                if default_provider.is_none() {
                    default_provider = Some("bedrock".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to initialize AWS Bedrock provider: {}", e);
            }
        }
    } else if std::env::var("HIVECODE_USE_BEDROCK").ok() == Some("1".to_string()) {
        // Try environment variable as fallback
        match BedrockProvider::builder()
            .region(
                std::env::var("AWS_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
            )
            .build()
        {
            Ok(provider) => {
                info!("AWS Bedrock provider initialized from environment variable");
                registry.register("bedrock".to_string(), Arc::new(provider))?;

                if default_provider.is_none() {
                    default_provider = Some("bedrock".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to build AWS Bedrock provider from env var: {}", e);
            }
        }
    }

    // Try to initialize Google Vertex AI provider
    if let Some(provider_config) = config.providers.get("vertex") {
        match initialize_vertex_provider(provider_config) {
            Ok(provider) => {
                info!("Google Vertex AI provider initialized successfully");
                registry.register("vertex".to_string(), Arc::new(provider))?;

                // Set as default only if no default is set yet
                if default_provider.is_none() {
                    default_provider = Some("vertex".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to initialize Google Vertex AI provider: {}", e);
            }
        }
    } else if std::env::var("HIVECODE_USE_VERTEX").ok() == Some("1".to_string()) {
        // Try environment variable as fallback
        if let Ok(project_id) = std::env::var("GOOGLE_CLOUD_PROJECT") {
            match VertexProvider::builder()
                .project_id(&project_id)
                .location(
                    std::env::var("GOOGLE_CLOUD_LOCATION")
                        .unwrap_or_else(|_| "us-central1".to_string()),
                )
                .build()
            {
                Ok(provider) => {
                    info!("Google Vertex AI provider initialized from environment variable");
                    registry.register("vertex".to_string(), Arc::new(provider))?;

                    if default_provider.is_none() {
                        default_provider = Some("vertex".to_string());
                    }
                }
                Err(e) => {
                    warn!("Failed to build Google Vertex AI provider from env var: {}", e);
                }
            }
        }
    }

    // Try to initialize Anthropic Foundry provider
    if let Some(provider_config) = config.providers.get("foundry") {
        match initialize_foundry_provider(provider_config) {
            Ok(provider) => {
                info!("Anthropic Foundry provider initialized successfully");
                registry.register("foundry".to_string(), Arc::new(provider))?;

                // Set as default only if no default is set yet
                if default_provider.is_none() {
                    default_provider = Some("foundry".to_string());
                }
            }
            Err(e) => {
                warn!("Failed to initialize Anthropic Foundry provider: {}", e);
            }
        }
    } else if std::env::var("HIVECODE_USE_FOUNDRY").ok() == Some("1".to_string()) {
        // Try environment variable as fallback
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let base_url = std::env::var("FOUNDRY_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

            match FoundryProvider::builder()
                .api_key(&api_key)
                .base_url(&base_url)
                .build()
            {
                Ok(provider) => {
                    info!("Anthropic Foundry provider initialized from environment variable");
                    registry.register("foundry".to_string(), Arc::new(provider))?;

                    if default_provider.is_none() {
                        default_provider = Some("foundry".to_string());
                    }
                }
                Err(e) => {
                    warn!("Failed to build Anthropic Foundry provider from env var: {}", e);
                }
            }
        }
    }

    // Set the default provider
    if let Some(default_name) = default_provider {
        registry.set_default(default_name)?;
        info!("Default provider set successfully");
    } else {
        warn!("No providers could be initialized - application may not function properly");
    }

    Ok(registry)
}

/// Initialize Anthropic provider from configuration
fn initialize_anthropic_provider(
    config: &hivecode_core::config::ProviderConfig,
) -> Result<AnthropicProvider> {
    let api_key = config
        .api_key
        .clone()
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .ok_or_else(|| {
            ProviderError::InvalidConfig("Anthropic API key not found in config or environment".to_string())
        })?;

    let mut builder = AnthropicProvider::builder().api_key(&api_key);

    // Set custom base URL if provided
    if let Some(base_url) = &config.base_url {
        builder = builder.base_url(base_url);
    }

    // Set default model if provided
    if let Some(default_model) = &config.default_model {
        builder = builder.default_model(default_model);
    }

    builder.build()
}

/// Initialize OpenAI provider from configuration
fn initialize_openai_provider(
    config: &hivecode_core::config::ProviderConfig,
) -> Result<OpenAiProvider> {
    let api_key = config
        .api_key
        .clone()
        .or_else(|| std::env::var("OPENAI_API_KEY").ok());

    let base_url = config
        .base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

    let default_model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    let mut builder = OpenAiProvider::builder()
        .base_url(&base_url)
        .default_model(&default_model);

    if let Some(key) = api_key {
        builder = builder.api_key(&key);
    }

    Ok(builder.build())
}

/// Initialize Ollama provider from configuration
///
/// Attempts to connect to Ollama at the configured or default location.
/// Returns Ok(None) if Ollama is not reachable, Ok(Some(provider)) if it is.
async fn initialize_ollama_provider(
    config: &hivecode_core::config::HiveConfig,
) -> Result<Option<OllamaProvider>> {
    let base_url = config
        .providers
        .get("ollama")
        .and_then(|c| c.base_url.clone())
        .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let default_model = config
        .providers
        .get("ollama")
        .and_then(|c| c.default_model.clone())
        .unwrap_or_else(|| "llama2".to_string());

    let provider = OllamaProvider::builder()
        .base_url(&base_url)
        .default_model(&default_model)
        .build();

    // Health check to see if Ollama is actually running
    match provider.health_check().await {
        Ok(true) => {
            debug!("Ollama health check passed");
            Ok(Some(provider))
        }
        Ok(false) => {
            debug!("Ollama is not reachable at {}", base_url);
            Ok(None)
        }
        Err(e) => {
            debug!("Ollama health check failed: {}", e);
            Ok(None)
        }
    }
}

/// Initialize AWS Bedrock provider from configuration
fn initialize_bedrock_provider(
    config: &hivecode_core::config::ProviderConfig,
) -> Result<BedrockProvider> {
    // Extract region from settings or env var
    let region = config
        .settings
        .get("region")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("AWS_REGION").ok())
        .unwrap_or_else(|| "us-east-1".to_string());

    let default_model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    let mut builder = BedrockProvider::builder()
        .region(&region)
        .default_model(&default_model);

    // Set credentials if provided in config
    if let (Some(access_key), Some(secret_key)) = (
        config
            .settings
            .get("access_key_id")
            .and_then(|v| v.as_str()),
        config
            .settings
            .get("secret_access_key")
            .and_then(|v| v.as_str()),
    ) {
        builder = builder.credentials(access_key, secret_key);
    }

    // Set session token if provided
    if let Some(token) = config
        .settings
        .get("session_token")
        .and_then(|v| v.as_str())
    {
        builder = builder.session_token(token);
    }

    builder.build()
}

/// Initialize Google Vertex AI provider from configuration
fn initialize_vertex_provider(
    config: &hivecode_core::config::ProviderConfig,
) -> Result<VertexProvider> {
    // Extract project ID from settings or env var
    let project_id = config
        .settings
        .get("project_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
        .ok_or_else(|| {
            ProviderError::InvalidConfig("Google Cloud project ID not found".to_string())
        })?;

    let location = config
        .settings
        .get("location")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| std::env::var("GOOGLE_CLOUD_LOCATION").ok())
        .unwrap_or_else(|| "us-central1".to_string());

    let default_model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    let mut builder = VertexProvider::builder()
        .project_id(&project_id)
        .location(&location)
        .default_model(&default_model);

    // Set credentials JSON path if provided
    if let Some(creds) = config
        .settings
        .get("credentials_json")
        .and_then(|v| v.as_str())
    {
        builder = builder.credentials_json(creds);
    }

    builder.build()
}

/// Initialize Anthropic Foundry provider from configuration
fn initialize_foundry_provider(
    config: &hivecode_core::config::ProviderConfig,
) -> Result<FoundryProvider> {
    let api_key = config
        .api_key
        .clone()
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .ok_or_else(|| {
            ProviderError::InvalidConfig(
                "Anthropic API key not found in config or environment".to_string(),
            )
        })?;

    let base_url = config
        .base_url
        .clone()
        .or_else(|| std::env::var("FOUNDRY_BASE_URL").ok())
        .unwrap_or_else(|| "https://api.anthropic.com".to_string());

    let default_model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

    FoundryProvider::builder()
        .api_key(&api_key)
        .base_url(&base_url)
        .default_model(&default_model)
        .build()
}
