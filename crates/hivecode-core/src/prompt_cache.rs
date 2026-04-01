//! Prompt caching for HiveCode
//!
//! Implements cache-aware message construction for providers that support it.
//! Anthropic's prompt caching can reduce costs by up to 90% on repeated context.

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// A block of content that can be cached
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheableBlock {
    /// The actual content
    pub content: String,
    /// Hash of content for cache key
    pub cache_key: String,
    /// Number of tokens in this block
    pub token_count: u32,
    /// Type of block being cached
    pub block_type: CacheBlockType,
    /// Cache control directives
    pub cache_control: Option<CacheControl>,
}

/// Types of cacheable blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheBlockType {
    /// System prompt text
    SystemPrompt,
    /// Project-specific instructions from HIVECODE.md
    ProjectInstructions,
    /// Content of frequently referenced files
    FileContent,
    /// Early conversation messages
    ConversationPrefix,
    /// Tool definitions and schemas
    ToolDefinitions,
}

/// Cache control directives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    /// Cache type (e.g., "ephemeral" for Anthropic)
    pub cache_type: String,
}

/// Statistics about cache performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of cache writes
    pub cache_writes: u64,
    /// Total tokens saved by caching
    pub tokens_saved: u64,
    /// Estimated cost savings in dollars
    pub cost_saved: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            cache_writes: 0,
            tokens_saved: 0,
            cost_saved: 0.0,
            hit_rate: 0.0,
        }
    }
}

/// Information about a cached entry
#[derive(Debug, Clone)]
struct CacheEntry {
    block: CacheableBlock,
    last_used: Instant,
    use_count: u64,
    created_at: Instant,
}

/// Manages prompt caching for conversation messages
pub struct PromptCacheManager {
    cache_entries: HashMap<String, CacheEntry>,
    stats: CacheStats,
    max_cache_size: usize,
    min_block_tokens: u32,
}

impl PromptCacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self {
            cache_entries: HashMap::new(),
            stats: CacheStats::default(),
            max_cache_size: 100,
            min_block_tokens: 1024,
        }
    }

    /// Set the maximum number of cache entries
    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_cache_size = max;
        self
    }

    /// Set the minimum block size to be worth caching (in tokens)
    pub fn with_min_tokens(mut self, min: u32) -> Self {
        self.min_block_tokens = min;
        self
    }

    /// Prepare messages with cache breakpoints for Anthropic API
    pub fn prepare_cached_messages(
        &mut self,
        messages: &[serde_json::Value],
    ) -> Vec<serde_json::Value> {
        let cacheable = self.identify_cacheable_blocks(messages);
        let mut result = messages.to_vec();

        self.add_cache_markers(&mut result, &cacheable);

        for block in &cacheable {
            self.cache_block(block);
        }

        result
    }

    /// Identify which parts of a message list are cacheable
    pub fn identify_cacheable_blocks(&self, messages: &[serde_json::Value]) -> Vec<CacheableBlock> {
        let mut blocks = Vec::new();

        for message in messages {
            if let Some(content) = message.get("content") {
                if let Some(text) = content.as_str() {
                    let token_count = self.estimate_tokens(text);
                    if token_count >= self.min_block_tokens {
                        let cache_key = self.hash_content(text);
                        blocks.push(CacheableBlock {
                            content: text.to_string(),
                            cache_key,
                            token_count,
                            block_type: CacheBlockType::SystemPrompt,
                            cache_control: Some(CacheControl {
                                cache_type: "ephemeral".to_string(),
                            }),
                        });
                    }
                }
            }
        }

        blocks
    }

    /// Add cache_control markers to message content blocks
    pub fn add_cache_markers(
        &self,
        messages: &mut [serde_json::Value],
        cacheable: &[CacheableBlock],
    ) {
        if cacheable.is_empty() {
            return;
        }

        // Mark the last cacheable block with cache control
        if let Some(last_block) = cacheable.last() {
            for message in messages.iter_mut().rev() {
                if let Some(content) = message.get("content") {
                    if let Some(text) = content.as_str() {
                        if text == &last_block.content {
                            if let Some(obj) = message.as_object_mut() {
                                obj.insert(
                                    "cache_control".to_string(),
                                    serde_json::json!({
                                        "type": "ephemeral"
                                    }),
                                );
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Record cache results from API response
    pub fn record_cache_result(&mut self, cache_creation_tokens: u64, cache_read_tokens: u64) {
        if cache_read_tokens > 0 {
            self.stats.cache_hits += 1;
            self.stats.tokens_saved += cache_read_tokens;

            // Estimate cost savings: Anthropic charges ~20% for cache reads vs regular input
            let cache_read_cost = cache_read_tokens as f64 * 0.0002; // Approximate per-token cost
            let regular_read_cost = cache_read_tokens as f64 * 0.001;
            self.stats.cost_saved += regular_read_cost - cache_read_cost;
        } else {
            self.stats.cache_misses += 1;
        }

        if cache_creation_tokens > 0 {
            self.stats.cache_writes += 1;
        }

        // Calculate hit rate
        let total = self.stats.cache_hits + self.stats.cache_misses;
        if total > 0 {
            self.stats.hit_rate = self.stats.cache_hits as f64 / total as f64;
        }

        debug!(
            "Cache stats - hits: {}, misses: {}, hit_rate: {:.2}%, tokens_saved: {}",
            self.stats.cache_hits,
            self.stats.cache_misses,
            self.stats.hit_rate * 100.0,
            self.stats.tokens_saved
        );
    }

    /// Get current cache statistics
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache_entries.clear();
        debug!("Cache cleared");
    }

    // Private helper methods

    fn cache_block(&mut self, block: &CacheableBlock) {
        if self.cache_entries.len() >= self.max_cache_size {
            self.evict_lru();
        }

        self.cache_entries.insert(
            block.cache_key.clone(),
            CacheEntry {
                block: block.clone(),
                last_used: Instant::now(),
                use_count: 0,
                created_at: Instant::now(),
            },
        );

        self.stats.cache_writes += 1;
        info!(
            "Cached block: {} tokens, total cache size: {}",
            block.token_count,
            self.cache_entries.len()
        );
    }

    fn hash_content(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn estimate_tokens(&self, content: &str) -> u32 {
        // Rough estimation: ~4 characters per token
        ((content.len() + 3) / 4) as u32
    }

    fn evict_lru(&mut self) {
        // Find the least recently used entry
        let lru_key = self
            .cache_entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            self.cache_entries.remove(&key);
            debug!("Evicted LRU cache entry: {}", key);
        }
    }
}

impl Default for PromptCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache_manager() {
        let manager = PromptCacheManager::new();
        assert_eq!(manager.cache_entries.len(), 0);
        assert_eq!(manager.stats.cache_hits, 0);
        assert_eq!(manager.stats.cache_misses, 0);
    }

    #[test]
    fn test_with_max_size() {
        let manager = PromptCacheManager::new().with_max_size(50);
        assert_eq!(manager.max_cache_size, 50);
    }

    #[test]
    fn test_with_min_tokens() {
        let manager = PromptCacheManager::new().with_min_tokens(512);
        assert_eq!(manager.min_block_tokens, 512);
    }

    #[test]
    fn test_hash_content() {
        let manager = PromptCacheManager::new();
        let hash1 = manager.hash_content("test content");
        let hash2 = manager.hash_content("test content");
        let hash3 = manager.hash_content("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_estimate_tokens() {
        let manager = PromptCacheManager::new();
        let count1 = manager.estimate_tokens("a");
        let count2 = manager.estimate_tokens("abcd");
        let count4 = manager.estimate_tokens("abcdefgh");

        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
        assert_eq!(count4, 2);
    }

    #[test]
    fn test_record_cache_hit() {
        let mut manager = PromptCacheManager::new();
        manager.record_cache_result(0, 1000);

        assert_eq!(manager.stats.cache_hits, 1);
        assert_eq!(manager.stats.cache_misses, 0);
        assert_eq!(manager.stats.tokens_saved, 1000);
        assert!(manager.stats.hit_rate > 0.0);
    }

    #[test]
    fn test_record_cache_miss() {
        let mut manager = PromptCacheManager::new();
        manager.record_cache_result(500, 0);

        assert_eq!(manager.stats.cache_hits, 0);
        assert_eq!(manager.stats.cache_misses, 1);
        assert_eq!(manager.stats.tokens_saved, 0);
        assert_eq!(manager.stats.hit_rate, 0.0);
    }

    #[test]
    fn test_identify_cacheable_blocks() {
        let manager = PromptCacheManager::new().with_min_tokens(100);
        let messages = vec![serde_json::json!({
            "role": "system",
            "content": "This is a system prompt with enough tokens to be cached. It should exceed the minimum token requirement."
        })];

        let blocks = manager.identify_cacheable_blocks(&messages);
        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].block_type, CacheBlockType::SystemPrompt);
    }

    #[test]
    fn test_identify_small_blocks_not_cached() {
        let manager = PromptCacheManager::new().with_min_tokens(1024);
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": "Hi"
        })];

        let blocks = manager.identify_cacheable_blocks(&messages);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_add_cache_markers() {
        let manager = PromptCacheManager::new();
        let cacheable = vec![CacheableBlock {
            content: "test content".to_string(),
            cache_key: "key".to_string(),
            token_count: 100,
            block_type: CacheBlockType::SystemPrompt,
            cache_control: Some(CacheControl {
                cache_type: "ephemeral".to_string(),
            }),
        }];

        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": "test content"
        })];

        manager.add_cache_markers(&mut messages, &cacheable);

        // Verify cache_control was added
        assert!(messages[0].get("cache_control").is_some());
    }

    #[test]
    fn test_clear_cache() {
        let mut manager = PromptCacheManager::new();
        let block = CacheableBlock {
            content: "test".to_string(),
            cache_key: "key".to_string(),
            token_count: 100,
            block_type: CacheBlockType::FileContent,
            cache_control: None,
        };

        manager.cache_entries.insert("key".to_string(), CacheEntry {
            block,
            last_used: Instant::now(),
            use_count: 0,
            created_at: Instant::now(),
        });

        assert!(!manager.cache_entries.is_empty());
        manager.clear();
        assert!(manager.cache_entries.is_empty());
    }

    #[test]
    fn test_cache_block_eviction() {
        let mut manager = PromptCacheManager::new().with_max_size(2);

        // Add 3 blocks to trigger eviction
        for i in 0..3 {
            let block = CacheableBlock {
                content: format!("content {}", i),
                cache_key: format!("key{}", i),
                token_count: 100,
                block_type: CacheBlockType::FileContent,
                cache_control: None,
            };

            manager.cache_entries.insert(
                format!("key{}", i),
                CacheEntry {
                    block,
                    last_used: Instant::now(),
                    use_count: 0,
                    created_at: Instant::now(),
                },
            );
        }

        // Should only have 2 entries due to eviction
        assert!(manager.cache_entries.len() <= 2);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let mut manager = PromptCacheManager::new();

        // Record 8 hits and 2 misses
        for _ in 0..8 {
            manager.record_cache_result(0, 100);
        }
        for _ in 0..2 {
            manager.record_cache_result(0, 0);
        }

        assert_eq!(manager.stats.cache_hits, 8);
        assert_eq!(manager.stats.cache_misses, 2);
        assert!((manager.stats.hit_rate - 0.8).abs() < 0.01); // Should be 0.8
    }

    #[test]
    fn test_cache_control_serialization() {
        let control = CacheControl {
            cache_type: "ephemeral".to_string(),
        };
        let json = serde_json::to_string(&control).unwrap();
        let restored: CacheControl = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.cache_type, "ephemeral");
    }

    #[test]
    fn test_cacheable_block_types() {
        let block_types = vec![
            CacheBlockType::SystemPrompt,
            CacheBlockType::ProjectInstructions,
            CacheBlockType::FileContent,
            CacheBlockType::ConversationPrefix,
            CacheBlockType::ToolDefinitions,
        ];

        for block_type in block_types {
            let json = serde_json::to_string(&block_type).unwrap();
            let _restored: CacheBlockType = serde_json::from_str(&json).unwrap();
            // Just verify it's serializable
        }
    }
}
