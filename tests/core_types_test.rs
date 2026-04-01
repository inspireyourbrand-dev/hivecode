//! Integration tests for HiveCode core type definitions
//!
//! Tests message creation, role enum, content blocks, token counting,
//! and provider info structures.

use hivecode_core::types::{
    ContentBlock, Message, MessageRole, ProviderInfo, TokenCount,
};
use std::collections::HashMap;

#[test]
fn test_message_role_display() {
    assert_eq!(MessageRole::User.to_string(), "user");
    assert_eq!(MessageRole::Assistant.to_string(), "assistant");
    assert_eq!(MessageRole::System.to_string(), "system");
    assert_eq!(MessageRole::Tool.to_string(), "tool");
}

#[test]
fn test_message_role_equality() {
    assert_eq!(MessageRole::User, MessageRole::User);
    assert_ne!(MessageRole::User, MessageRole::Assistant);
    assert_ne!(MessageRole::System, MessageRole::Tool);
}

#[test]
fn test_message_role_copy() {
    let role = MessageRole::User;
    let role_copy = role;
    assert_eq!(role, role_copy);
}

#[test]
fn test_content_block_text() {
    let block = ContentBlock::Text {
        content: "Hello, world!".to_string(),
    };

    match &block {
        ContentBlock::Text { content } => {
            assert_eq!(content, "Hello, world!");
        }
        _ => panic!("Expected Text variant"),
    }
}

#[test]
fn test_content_block_as_text_some() {
    let block = ContentBlock::Text {
        content: "Test content".to_string(),
    };

    let text = block.as_text();
    assert!(text.is_some());
    assert_eq!(text.unwrap(), "Test content");
}

#[test]
fn test_content_block_as_text_none_for_tool_use() {
    let block = ContentBlock::ToolUse {
        id: "tool_1".to_string(),
        name: "search".to_string(),
        input: serde_json::json!({"query": "test"}),
    };

    let text = block.as_text();
    assert!(text.is_none());
}

#[test]
fn test_content_block_as_text_none_for_tool_result() {
    let block = ContentBlock::ToolResult {
        tool_use_id: "tool_1".to_string(),
        content: "Result content".to_string(),
        is_error: false,
    };

    let text = block.as_text();
    assert!(text.is_none());
}

#[test]
fn test_content_block_tool_use() {
    let input = serde_json::json!({
        "query": "test query",
        "limit": 10
    });

    let block = ContentBlock::ToolUse {
        id: "tool_123".to_string(),
        name: "web_search".to_string(),
        input,
    };

    match block {
        ContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "tool_123");
            assert_eq!(name, "web_search");
            assert_eq!(input["query"].as_str(), Some("test query"));
            assert_eq!(input["limit"].as_u64(), Some(10));
        }
        _ => panic!("Expected ToolUse variant"),
    }
}

#[test]
fn test_content_block_tool_result_success() {
    let block = ContentBlock::ToolResult {
        tool_use_id: "tool_123".to_string(),
        content: "Search results here".to_string(),
        is_error: false,
    };

    match block {
        ContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            assert_eq!(tool_use_id, "tool_123");
            assert_eq!(content, "Search results here");
            assert!(!is_error);
        }
        _ => panic!("Expected ToolResult variant"),
    }
}

#[test]
fn test_content_block_tool_result_error() {
    let block = ContentBlock::ToolResult {
        tool_use_id: "tool_456".to_string(),
        content: "Tool execution failed".to_string(),
        is_error: true,
    };

    match block {
        ContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            assert_eq!(tool_use_id, "tool_456");
            assert_eq!(content, "Tool execution failed");
            assert!(is_error);
        }
        _ => panic!("Expected ToolResult variant"),
    }
}

#[test]
fn test_message_new() {
    let content = vec![ContentBlock::Text {
        content: "Hello".to_string(),
    }];

    let message = Message::new(MessageRole::User, content);

    assert_eq!(message.role, MessageRole::User);
    assert_eq!(message.content.len(), 1);
    assert!(message.parent_id.is_none());
    assert!(message.tokens.is_none());
    // ID should be a valid UUID
    assert!(!message.id.is_empty());
}

#[test]
fn test_message_text() {
    let message = Message::text(MessageRole::User, "Hello, world!");

    assert_eq!(message.role, MessageRole::User);
    assert_eq!(message.content.len(), 1);
    assert_eq!(message.content[0].as_text(), Some("Hello, world!"));
}

#[test]
fn test_message_text_with_string() {
    let text_string = String::from("Test message");
    let message = Message::text(MessageRole::Assistant, text_string);

    assert_eq!(message.role, MessageRole::Assistant);
    assert_eq!(message.content[0].as_text(), Some("Test message"));
}

#[test]
fn test_message_get_text_single_block() {
    let message = Message::text(MessageRole::User, "Single line");

    let combined = message.get_text();
    assert_eq!(combined, "Single line");
}

#[test]
fn test_message_get_text_multiple_blocks() {
    let content = vec![
        ContentBlock::Text {
            content: "First".to_string(),
        },
        ContentBlock::Text {
            content: "Second".to_string(),
        },
        ContentBlock::ToolUse {
            id: "tool_1".to_string(),
            name: "search".to_string(),
            input: serde_json::json!({}),
        },
        ContentBlock::Text {
            content: "Third".to_string(),
        },
    ];

    let message = Message::new(MessageRole::Assistant, content);
    let combined = message.get_text();

    // Should only include Text blocks
    assert!(combined.contains("First"));
    assert!(combined.contains("Second"));
    assert!(combined.contains("Third"));
}

#[test]
fn test_message_get_text_with_newlines() {
    let content = vec![
        ContentBlock::Text {
            content: "Line 1".to_string(),
        },
        ContentBlock::Text {
            content: "Line 2".to_string(),
        },
    ];

    let message = Message::new(MessageRole::User, content);
    let combined = message.get_text();

    assert_eq!(combined, "Line 1\nLine 2");
}

#[test]
fn test_token_count_new() {
    let tc = TokenCount::new(100, 50);

    assert_eq!(tc.input, 100);
    assert_eq!(tc.output, 50);
    assert_eq!(tc.total, 150);
}

#[test]
fn test_token_count_default() {
    let tc = TokenCount::default();

    assert_eq!(tc.input, 0);
    assert_eq!(tc.output, 0);
    assert_eq!(tc.total, 0);
}

#[test]
fn test_token_count_add() {
    let mut tc1 = TokenCount::new(100, 50);
    let tc2 = TokenCount::new(75, 25);

    tc1.add(&tc2);

    assert_eq!(tc1.input, 175);
    assert_eq!(tc1.output, 75);
    assert_eq!(tc1.total, 250);
}

#[test]
fn test_token_count_serialization() {
    let tc = TokenCount::new(1000, 500);
    let json = serde_json::to_string(&tc).expect("Failed to serialize");
    let restored: TokenCount =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(tc.input, restored.input);
    assert_eq!(tc.output, restored.output);
    assert_eq!(tc.total, restored.total);
}

#[test]
fn test_message_serialization() {
    let message = Message::text(MessageRole::User, "Test message");
    let json = serde_json::to_string(&message).expect("Failed to serialize");
    let restored: Message =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(message.role, restored.role);
    assert_eq!(message.get_text(), restored.get_text());
}

#[test]
fn test_provider_info_creation() {
    let provider = ProviderInfo {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        available: true,
        models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        config: None,
    };

    assert_eq!(provider.id, "openai");
    assert_eq!(provider.name, "OpenAI");
    assert!(provider.available);
    assert_eq!(provider.models.len(), 2);
    assert!(provider.config.is_none());
}

#[test]
fn test_provider_info_with_config() {
    let mut config = HashMap::new();
    config.insert(
        "api_version".to_string(),
        serde_json::json!("2024-01-01"),
    );

    let provider = ProviderInfo {
        id: "anthropic".to_string(),
        name: "Anthropic".to_string(),
        available: true,
        models: vec!["claude-3".to_string()],
        config: Some(config),
    };

    assert!(provider.config.is_some());
    let cfg = provider.config.unwrap();
    assert_eq!(cfg.get("api_version").unwrap().as_str(), Some("2024-01-01"));
}

#[test]
fn test_provider_info_unavailable() {
    let provider = ProviderInfo {
        id: "local".to_string(),
        name: "Local Model".to_string(),
        available: false,
        models: vec![],
        config: None,
    };

    assert!(!provider.available);
    assert!(provider.models.is_empty());
}

#[test]
fn test_provider_info_serialization() {
    let provider = ProviderInfo {
        id: "test".to_string(),
        name: "Test Provider".to_string(),
        available: true,
        models: vec!["model1".to_string(), "model2".to_string()],
        config: None,
    };

    let json = serde_json::to_string(&provider).expect("Failed to serialize");
    let restored: ProviderInfo =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(provider.id, restored.id);
    assert_eq!(provider.name, restored.name);
    assert_eq!(provider.available, restored.available);
    assert_eq!(provider.models, restored.models);
}

#[test]
fn test_message_with_parent_id() {
    let mut message = Message::text(MessageRole::User, "Reply to parent");
    message.parent_id = Some("parent_msg_123".to_string());

    assert_eq!(message.parent_id, Some("parent_msg_123".to_string()));
}

#[test]
fn test_message_with_token_count() {
    let mut message = Message::text(MessageRole::Assistant, "Response");
    message.tokens = Some(TokenCount::new(500, 1000));

    assert!(message.tokens.is_some());
    let tokens = message.tokens.unwrap();
    assert_eq!(tokens.input, 500);
    assert_eq!(tokens.output, 1000);
}

#[test]
fn test_content_block_empty_text() {
    let block = ContentBlock::Text {
        content: "".to_string(),
    };

    assert_eq!(block.as_text(), Some(""));
}

#[test]
fn test_message_get_text_empty_message() {
    let message = Message::new(MessageRole::User, vec![]);
    let combined = message.get_text();
    assert_eq!(combined, "");
}

#[test]
fn test_message_unique_ids() {
    let msg1 = Message::text(MessageRole::User, "msg1");
    let msg2 = Message::text(MessageRole::User, "msg2");

    assert_ne!(msg1.id, msg2.id, "Messages should have unique IDs");
}

#[test]
fn test_token_count_zero() {
    let tc = TokenCount {
        input: 0,
        output: 0,
        total: 0,
    };

    assert_eq!(tc.total, 0);
}

#[test]
fn test_content_block_clone() {
    let original = ContentBlock::Text {
        content: "Clone me".to_string(),
    };
    let cloned = original.clone();

    assert_eq!(original.as_text(), cloned.as_text());
}

#[test]
fn test_message_multiple_content_types() {
    let content = vec![
        ContentBlock::Text {
            content: "User asks question".to_string(),
        },
        ContentBlock::ToolUse {
            id: "search_1".to_string(),
            name: "search".to_string(),
            input: serde_json::json!({"q": "question"}),
        },
        ContentBlock::ToolResult {
            tool_use_id: "search_1".to_string(),
            content: "Answer found".to_string(),
            is_error: false,
        },
        ContentBlock::Text {
            content: "Here is the answer".to_string(),
        },
    ];

    let message = Message::new(MessageRole::Assistant, content);

    assert_eq!(message.content.len(), 4);
    assert_eq!(message.get_text(), "User asks question\nHere is the answer");
}
