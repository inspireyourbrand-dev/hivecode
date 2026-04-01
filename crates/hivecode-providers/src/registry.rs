//! Provider registry for managing multiple LLM providers.

use crate::traits::LlmProvider;
use crate::{ProviderError, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing multiple LLM providers.
///
/// This allows registration, lookup, and switching between different LLM providers.
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn LlmProvider>>>,
    default: RwLock<Option<String>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default: RwLock::new(None),
        }
    }

    /// Register a new provider
    ///
    /// # Arguments
    ///
    /// * `name` - Unique name for this provider instance
    /// * `provider` - The provider implementation
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use hivecode_providers::ProviderRegistry;
    /// # use std::sync::Arc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = ProviderRegistry::new();
    /// # /*
    /// registry.register("openai".to_string(), Arc::new(openai_provider));
    /// # */
    /// # Ok(())
    /// # }
    /// ```
    pub fn register(&self, name: String, provider: Arc<dyn LlmProvider>) -> Result<()> {
        let mut providers = self.providers.write().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire write lock: {}", e))
        })?;
        providers.insert(name, provider);
        Ok(())
    }

    /// Get a provider by name
    ///
    /// # Arguments
    ///
    /// * `name` - The provider name
    ///
    /// # Returns
    ///
    /// The provider, or an error if not found
    pub fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>> {
        let providers = self.providers.read().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire read lock: {}", e))
        })?;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| ProviderError::Other(format!("Provider '{}' not found", name)))
    }

    /// Get the default provider
    ///
    /// # Returns
    ///
    /// The default provider, or an error if none is set
    pub fn get_default(&self) -> Result<Arc<dyn LlmProvider>> {
        let default = self.default.read().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire read lock: {}", e))
        })?;
        let name = default
            .as_ref()
            .ok_or_else(|| ProviderError::Other("No default provider set".to_string()))?;
        self.get(name)
    }

    /// Set the default provider
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the provider to set as default
    pub fn set_default(&self, name: String) -> Result<()> {
        // Verify the provider exists
        self.get(&name)?;

        let mut default = self.default.write().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire write lock: {}", e))
        })?;
        *default = Some(name);
        Ok(())
    }

    /// List all registered providers
    ///
    /// # Returns
    ///
    /// Vector of provider names
    pub fn list(&self) -> Result<Vec<String>> {
        let providers = self.providers.read().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(providers.keys().cloned().collect())
    }

    /// Remove a provider
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the provider to remove
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut providers = self.providers.write().map_err(|e| {
            ProviderError::Other(format!("Failed to acquire write lock: {}", e))
        })?;
        providers.remove(name);
        Ok(())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = ProviderRegistry::new();
        assert!(registry.list().unwrap().is_empty());
        assert!(registry.get_default().is_err());
    }
}
