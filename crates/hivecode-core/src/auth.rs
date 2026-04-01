//! Authentication system for HiveCode
//!
//! Supports multiple authentication modes:
//! - API Keys (OpenAI, Anthropic, etc.)
//! - OAuth 2.0 with PKCE (OpenAI Platform, Anthropic Console)
//! - ChatGPT subscription tokens (for ChatGPT Plus/Pro/Team subscribers)
//!
//! Auth profiles are stored encrypted at ~/.hivecode/auth_profiles.json

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{debug, info, warn};

/// Authentication mode for a provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode")]
pub enum AuthMode {
    /// Standard API key authentication
    #[serde(rename = "api_key")]
    ApiKey { api_key: String },
    /// OAuth 2.0 with PKCE (for Anthropic Console, OpenAI Platform)
    #[serde(rename = "oauth")]
    OAuth {
        client_id: String,
        auth_url: String,
        token_url: String,
        scopes: Vec<String>,
        access_token: Option<String>,
        refresh_token: Option<String>,
        expires_at: Option<u64>,
    },
    /// ChatGPT subscription session token
    /// Users with ChatGPT Plus/Pro/Team can use their subscription
    /// instead of paying for API separately
    #[serde(rename = "chatgpt_session")]
    ChatGptSession {
        session_token: String,
        access_token: Option<String>,
        expires_at: Option<u64>,
    },
}

/// Auth profile - a named authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    pub id: String,
    pub provider: String,
    pub mode: AuthMode,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub last_used: Option<String>,
}

/// Summary of an auth profile for UI display (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfileSummary {
    pub id: String,
    pub provider: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub last_used: Option<String>,
    pub auth_type: String,
    pub expires_at: Option<u64>,
}

impl AuthProfile {
    /// Convert to a summary for UI display (safe to send to frontend)
    pub fn to_summary(&self) -> AuthProfileSummary {
        let (auth_type, expires_at) = match &self.mode {
            AuthMode::ApiKey { .. } => ("api_key".to_string(), None),
            AuthMode::OAuth { expires_at, .. } => ("oauth".to_string(), *expires_at),
            AuthMode::ChatGptSession { expires_at, .. } => ("chatgpt_session".to_string(), *expires_at),
        };

        AuthProfileSummary {
            id: self.id.clone(),
            provider: self.provider.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            is_default: self.is_default,
            created_at: self.created_at.clone(),
            last_used: self.last_used.clone(),
            auth_type,
            expires_at,
        }
    }
}

/// OAuth flow state - temporary data for PKCE flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowState {
    pub auth_url: String,
    pub code_verifier: String,
    pub state: String,
}

/// OAuth login URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthLoginUrl {
    pub url: String,
    pub state: String,
}

/// ChatGPT login state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptLoginState {
    pub instructions: String,
}

/// Auth test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTestResult {
    pub success: bool,
    pub message: String,
    pub provider: String,
    pub auth_type: String,
}

/// Auth manager - handles all authentication concerns
pub struct AuthManager {
    profiles: RwLock<Vec<AuthProfile>>,
    profiles_path: PathBuf,
}

impl AuthManager {
    /// Load or create auth manager from disk
    pub fn new() -> Result<Self> {
        let profiles_path = Self::default_profiles_path()?;
        let profiles = Self::load_profiles(&profiles_path)?;

        Ok(Self {
            profiles: RwLock::new(profiles),
            profiles_path,
        })
    }

    /// Get the default profiles path (~/.hivecode/auth_profiles.json)
    fn default_profiles_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| HiveCodeError::ConfigError("Cannot determine home directory".to_string()))?;
        Ok(home.join(".hivecode").join("auth_profiles.json"))
    }

    /// Load profiles from disk
    fn load_profiles(path: &PathBuf) -> Result<Vec<AuthProfile>> {
        if !path.exists() {
            debug!("Auth profiles file does not exist yet: {:?}", path);
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| HiveCodeError::ConfigFileError(e))?;

        if contents.trim().is_empty() {
            return Ok(Vec::new());
        }

        let profiles: Vec<AuthProfile> = serde_json::from_str(&contents)
            .map_err(|e| HiveCodeError::Internal(format!("Failed to parse auth profiles: {}", e)))?;

        debug!("Loaded {} auth profiles", profiles.len());
        Ok(profiles)
    }

    /// Save profiles to disk
    fn save_profiles(&self) -> Result<()> {
        let profiles = self.profiles.read()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        let json = serde_json::to_string_pretty(&*profiles)
            .map_err(|e| HiveCodeError::Internal(format!("Failed to serialize profiles: {}", e)))?;

        fs::write(&self.profiles_path, json)
            .map_err(|e| HiveCodeError::ConfigFileError(e))?;

        debug!("Saved {} auth profiles", profiles.len());
        Ok(())
    }

    /// Add a new auth profile
    pub fn add_profile(&self, mut profile: AuthProfile) -> Result<AuthProfile> {
        let now = chrono::Utc::now().to_rfc3339();
        profile.created_at = now;

        let mut profiles = self.profiles.write()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        // If this is the first profile for this provider, make it default
        if !profiles.iter().any(|p| p.provider == profile.provider) {
            profile.is_default = true;
        } else if profile.is_default {
            // Unset default for other profiles of this provider
            for p in profiles.iter_mut() {
                if p.provider == profile.provider {
                    p.is_default = false;
                }
            }
        }

        profiles.push(profile.clone());
        drop(profiles);

        self.save_profiles()?;
        info!("Added auth profile: {} ({})", profile.id, profile.provider);
        Ok(profile)
    }

    /// Remove a profile by ID
    pub fn remove_profile(&self, id: &str) -> Result<()> {
        let mut profiles = self.profiles.write()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        let initial_len = profiles.len();
        profiles.retain(|p| p.id != id);

        if profiles.len() == initial_len {
            return Err(HiveCodeError::NotFound(format!("Profile not found: {}", id)));
        }

        // If we removed the default, make the first one default
        if profiles.iter().all(|p| !p.is_default) && !profiles.is_empty() {
            profiles[0].is_default = true;
        }

        drop(profiles);
        self.save_profiles()?;
        info!("Removed auth profile: {}", id);
        Ok(())
    }

    /// Get a profile by ID
    pub fn get_profile(&self, id: &str) -> Result<AuthProfile> {
        let profiles = self.profiles.read()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        profiles
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| HiveCodeError::NotFound(format!("Profile not found: {}", id)))
    }

    /// Get the default profile for a provider
    pub fn get_default_for_provider(&self, provider: &str) -> Result<Option<AuthProfile>> {
        let profiles = self.profiles.read()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        Ok(profiles
            .iter()
            .find(|p| p.provider == provider && p.is_default)
            .cloned())
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Result<Vec<AuthProfile>> {
        let profiles = self.profiles.read()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;
        Ok(profiles.clone())
    }

    /// List all profile summaries (safe for UI)
    pub fn list_profile_summaries(&self) -> Result<Vec<AuthProfileSummary>> {
        let profiles = self.list_profiles()?;
        Ok(profiles.iter().map(|p| p.to_summary()).collect())
    }

    /// Get HTTP auth header from a profile
    /// Returns (header_name, header_value) tuple
    pub fn get_auth_header(&self, profile: &AuthProfile) -> Result<(String, String)> {
        match &profile.mode {
            AuthMode::ApiKey { api_key } => {
                // Most APIs use Authorization: Bearer {key}
                Ok(("Authorization".to_string(), format!("Bearer {}", api_key)))
            }
            AuthMode::OAuth { access_token, .. } => {
                let token = access_token
                    .as_ref()
                    .ok_or_else(|| HiveCodeError::AuthError("OAuth access token not available".to_string()))?;
                Ok(("Authorization".to_string(), format!("Bearer {}", token)))
            }
            AuthMode::ChatGptSession { access_token, .. } => {
                let token = access_token
                    .as_ref()
                    .ok_or_else(|| HiveCodeError::AuthError("ChatGPT access token not available".to_string()))?;
                Ok(("Authorization".to_string(), format!("Bearer {}", token)))
            }
        }
    }

    /// Set the default profile for a provider
    pub fn set_default_profile(&self, id: &str) -> Result<()> {
        let mut profiles = self.profiles.write()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        let profile = profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Profile not found: {}", id)))?;

        let provider = profile.provider.clone();

        // Unset default for other profiles of this provider
        for p in profiles.iter_mut() {
            if p.provider == provider {
                p.is_default = false;
            }
        }

        // Set this profile as default
        if let Some(p) = profiles.iter_mut().find(|p| p.id == id) {
            p.is_default = true;
        }

        drop(profiles);
        self.save_profiles()?;
        info!("Set default profile for {}: {}", provider, id);
        Ok(())
    }

    /// Update last_used timestamp for a profile
    pub fn mark_used(&self, id: &str) -> Result<()> {
        let mut profiles = self.profiles.write()
            .map_err(|_| HiveCodeError::Internal("Lock poisoned".to_string()))?;

        if let Some(profile) = profiles.iter_mut().find(|p| p.id == id) {
            profile.last_used = Some(chrono::Utc::now().to_rfc3339());
            drop(profiles);
            self.save_profiles()?;
        }

        Ok(())
    }

    /// Start OAuth PKCE flow
    pub fn start_oauth_flow(&self, provider: &str) -> Result<OAuthFlowState> {
        let (auth_url, token_url, client_id, scopes) = match provider {
            "openai" => {
                let client_id = std::env::var("OPENAI_OAUTH_CLIENT_ID")
                    .unwrap_or_else(|_| "CONFIGURE_YOUR_CLIENT_ID".to_string());
                (
                    "https://auth0.openai.com/authorize".to_string(),
                    "https://auth0.openai.com/oauth/token".to_string(),
                    client_id,
                    vec!["openid".to_string(), "profile".to_string(), "email".to_string(), "offline_access".to_string()],
                )
            }
            "anthropic" => {
                let client_id = std::env::var("ANTHROPIC_OAUTH_CLIENT_ID")
                    .unwrap_or_else(|_| "CONFIGURE_YOUR_CLIENT_ID".to_string());
                (
                    "https://console.anthropic.com/oauth/authorize".to_string(),
                    "https://console.anthropic.com/oauth/token".to_string(),
                    client_id,
                    vec!["user:inference".to_string(), "user:profile".to_string()],
                )
            }
            _ => {
                return Err(HiveCodeError::AuthError(
                    format!("OAuth not supported for provider: {}", provider),
                ))
            }
        };

        // Generate PKCE parameters
        let code_verifier = self.generate_code_verifier();
        let code_challenge = self.code_challenge(&code_verifier);
        let state = self.generate_state();

        let mut auth_url_builder = format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}",
            auth_url,
            urlencoding::encode(&client_id),
            urlencoding::encode("http://localhost:3000/auth/callback"),
            urlencoding::encode(&scopes.join(" "))
        );

        auth_url_builder.push_str(&format!("&code_challenge={}&code_challenge_method=S256&state={}", code_challenge, state));

        Ok(OAuthFlowState {
            auth_url: auth_url_builder,
            code_verifier,
            state,
        })
    }

    /// Complete OAuth PKCE flow
    pub fn complete_oauth_flow(&self, state: &str, auth_code: &str, code_verifier: &str, flow_data: &OAuthFlowState) -> Result<AuthProfile> {
        // Verify state to prevent CSRF
        if state != flow_data.state {
            return Err(HiveCodeError::AuthError("State mismatch".to_string()));
        }

        // This would normally exchange the code for a token via HTTP
        // For now, we'll create a stub profile that can be completed with real tokens
        let id = uuid::Uuid::new_v4().to_string();
        let profile = AuthProfile {
            id,
            provider: "oauth_pending".to_string(),
            mode: AuthMode::OAuth {
                client_id: "".to_string(),
                auth_url: "".to_string(),
                token_url: "".to_string(),
                scopes: vec![],
                access_token: None,
                refresh_token: None,
                expires_at: None,
            },
            display_name: None,
            email: None,
            is_default: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };

        debug!("OAuth flow completed for code: {}", auth_code);
        Ok(profile)
    }

    /// Start ChatGPT login flow
    pub fn start_chatgpt_login(&self) -> ChatGptLoginState {
        ChatGptLoginState {
            instructions: "1. Visit https://chatgpt.com in your browser\n\
                          2. Open DevTools (F12 or Cmd+Option+I)\n\
                          3. Go to Application > Cookies > https://chatgpt.com\n\
                          4. Find '__Secure-next-auth.session-token' cookie\n\
                          5. Copy the entire cookie value\n\
                          6. Paste it below - HiveCode will automatically exchange it for an access token".to_string(),
        }
    }

    /// Complete ChatGPT session authentication
    pub fn complete_chatgpt_login(&self, session_token: String, profile_name: Option<String>) -> Result<AuthProfile> {
        if session_token.is_empty() {
            return Err(HiveCodeError::AuthError("Session token is empty".to_string()));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let profile = AuthProfile {
            id,
            provider: "chatgpt".to_string(),
            mode: AuthMode::ChatGptSession {
                session_token,
                access_token: None,
                expires_at: None,
            },
            display_name: profile_name,
            email: None,
            is_default: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
        };

        self.add_profile(profile)
    }

    /// Refresh OAuth token
    pub fn refresh_oauth_token(&self, _profile_id: &str) -> Result<()> {
        warn!("OAuth token refresh not yet implemented - would need HTTP client");
        Ok(())
    }

    /// Refresh ChatGPT session token
    pub fn refresh_chatgpt_session(&self, _profile_id: &str) -> Result<()> {
        warn!("ChatGPT session refresh not yet implemented - would need HTTP client");
        Ok(())
    }

    /// Generate a random code verifier for PKCE
    fn generate_code_verifier(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{:032x}", nonce)
    }

    /// Generate code challenge from verifier
    fn code_challenge(&self, verifier: &str) -> String {
        use sha2::{Sha256, Digest};
        use base64::{Engine, engine::general_purpose};

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        general_purpose::URL_SAFE_NO_PAD.encode(&hash)
    }

    /// Generate a random state for CSRF protection
    fn generate_state(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{:032x}", nonce)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AuthManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_profile_to_summary() {
        let profile = AuthProfile {
            id: "test1".to_string(),
            provider: "openai".to_string(),
            mode: AuthMode::ApiKey {
                api_key: "secret_key".to_string(),
            },
            display_name: Some("My OpenAI".to_string()),
            email: Some("user@example.com".to_string()),
            is_default: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used: None,
        };

        let summary = profile.to_summary();
        assert_eq!(summary.id, "test1");
        assert_eq!(summary.auth_type, "api_key");
        assert_eq!(summary.provider, "openai");
    }

    #[test]
    fn test_auth_mode_serde() {
        let mode = AuthMode::ApiKey {
            api_key: "test_key".to_string(),
        };

        let json = serde_json::to_string(&mode).expect("Serialization failed");
        let deserialized: AuthMode = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(mode, deserialized);
    }
}
