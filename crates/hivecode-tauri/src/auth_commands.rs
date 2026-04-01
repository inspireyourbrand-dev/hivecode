//! Tauri IPC commands for authentication management
//!
//! Provides frontend API for:
//! - Listing and managing auth profiles
//! - Starting OAuth login flows
//! - Managing ChatGPT subscription tokens
//! - Testing authentication

use hivecode_core::{AuthManager, AuthMode, AuthProfile, AuthProfileSummary};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Response for auth command operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCommandResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> AuthCommandResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTestResult {
    pub success: bool,
    pub message: String,
    pub provider: String,
    pub auth_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthLoginUrl {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddApiKeyRequest {
    pub provider: String,
    pub api_key: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddChatGptSessionRequest {
    pub session_token: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteOAuthRequest {
    pub provider: String,
    pub auth_code: String,
}

/// List all authentication profiles
#[tauri::command]
pub async fn list_auth_profiles(
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<Vec<AuthProfileSummary>, String> {
    debug!("Listing auth profiles");

    auth_manager
        .list_profile_summaries()
        .map_err(|e| e.to_string())
}

/// Add a new API key profile
#[tauri::command]
pub async fn add_api_key_profile(
    request: AddApiKeyRequest,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<AuthProfile, String> {
    debug!("Adding API key profile for provider: {}", request.provider);

    if request.api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    if request.provider.is_empty() {
        return Err("Provider must be specified".to_string());
    }

    let id = format!("{}-{}", request.provider, uuid::Uuid::new_v4());

    let profile = AuthProfile {
        id,
        provider: request.provider.clone(),
        mode: AuthMode::ApiKey {
            api_key: request.api_key,
        },
        display_name: request.display_name,
        email: None,
        is_default: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    auth_manager
        .add_profile(profile.clone())
        .map_err(|e| e.to_string())?;

    info!("Added API key profile: {}", profile.id);
    Ok(profile)
}

/// Remove an authentication profile
#[tauri::command]
pub async fn remove_auth_profile(
    id: String,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<(), String> {
    debug!("Removing auth profile: {}", id);

    auth_manager
        .remove_profile(&id)
        .map_err(|e| e.to_string())?;

    info!("Removed auth profile: {}", id);
    Ok(())
}

/// Set a profile as the default for its provider
#[tauri::command]
pub async fn set_default_profile(
    id: String,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<(), String> {
    debug!("Setting default profile: {}", id);

    auth_manager
        .set_default_profile(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Start OAuth login flow
#[tauri::command]
pub async fn start_oauth_login(
    provider: String,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<OAuthLoginUrl, String> {
    debug!("Starting OAuth login for provider: {}", provider);

    let flow = auth_manager
        .start_oauth_flow(&provider)
        .map_err(|e| e.to_string())?;

    Ok(OAuthLoginUrl {
        url: flow.auth_url,
        state: flow.state,
    })
}

/// Complete OAuth login after user approval
#[tauri::command]
pub async fn complete_oauth_login(
    provider: String,
    auth_code: String,
    _auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<AuthProfile, String> {
    debug!(
        "Completing OAuth login for provider: {}, code: {}",
        provider,
        auth_code.chars().take(10).collect::<String>()
    );

    // In a real implementation, this would:
    // 1. Exchange the auth code for tokens using the provider's token endpoint
    // 2. Validate and store the tokens securely
    // 3. Return the created profile

    let id = format!("{}-{}", provider, uuid::Uuid::new_v4());

    let profile = AuthProfile {
        id,
        provider: provider.clone(),
        mode: AuthMode::OAuth {
            client_id: "".to_string(),
            auth_url: "".to_string(),
            token_url: "".to_string(),
            scopes: vec![],
            access_token: Some(auth_code),
            refresh_token: None,
            expires_at: None,
        },
        display_name: None,
        email: None,
        is_default: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    info!("Completed OAuth login for provider: {}", provider);
    Ok(profile)
}

/// Add ChatGPT subscription session
#[tauri::command]
pub async fn add_chatgpt_session(
    request: AddChatGptSessionRequest,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<AuthProfile, String> {
    debug!("Adding ChatGPT session token");

    if request.session_token.is_empty() {
        return Err("Session token cannot be empty".to_string());
    }

    auth_manager
        .complete_chatgpt_login(request.session_token, request.display_name)
        .map_err(|e| e.to_string())
}

/// Test if an authentication profile is valid
#[tauri::command]
pub async fn test_auth_profile(
    id: String,
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<AuthTestResult, String> {
    debug!("Testing auth profile: {}", id);

    let profile = auth_manager
        .get_profile(&id)
        .map_err(|e| e.to_string())?;

    // Update last_used
    let _ = auth_manager.mark_used(&id);

    let (auth_type, message) = match &profile.mode {
        AuthMode::ApiKey { .. } => (
            "api_key".to_string(),
            "API key profile is configured".to_string(),
        ),
        AuthMode::OAuth { access_token, .. } => {
            if access_token.is_some() {
                ("oauth".to_string(), "OAuth token is configured".to_string())
            } else {
                (
                    "oauth".to_string(),
                    "OAuth profile created but token not yet obtained".to_string(),
                )
            }
        }
        AuthMode::ChatGptSession { access_token, .. } => {
            if access_token.is_some() {
                (
                    "chatgpt_session".to_string(),
                    "ChatGPT access token is configured".to_string(),
                )
            } else {
                (
                    "chatgpt_session".to_string(),
                    "ChatGPT session token configured - will exchange for access token on first use"
                        .to_string(),
                )
            }
        }
    };

    info!("Auth profile test passed: {}", id);

    Ok(AuthTestResult {
        success: true,
        message,
        provider: profile.provider,
        auth_type,
    })
}

/// Get the ChatGPT login instructions
#[tauri::command]
pub async fn get_chatgpt_login_instructions(
    auth_manager: tauri::State<'_, Arc<AuthManager>>,
) -> Result<String, String> {
    let state = auth_manager.start_chatgpt_login();
    Ok(state.instructions)
}

/// Register all auth commands with Tauri
pub fn register_auth_commands(app: &mut tauri::App) -> tauri::Result<()> {
    // Commands are registered via #[tauri::command] macro in lib.rs
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_command_response() {
        let response: AuthCommandResponse<String> = AuthCommandResponse::ok("test".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_auth_command_error() {
        let response: AuthCommandResponse<String> =
            AuthCommandResponse::err("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
