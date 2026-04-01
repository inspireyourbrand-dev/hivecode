# HiveCode Test Suite

This directory contains integration tests for the HiveCode project. The test suite validates core functionality across three main areas: configuration, types, and security.

## Test Files

### core_config_test.rs
**23 tests** covering the configuration system (`hivecode-core::config`):

- **Default Values**: Verify that `AppConfig`, `SecurityConfig`, `FileAccessConfig`, `NetworkAccessConfig`, and `UiConfig` create correct defaults
- **Configuration Structures**: Test `HiveConfig`, `ProviderConfig`, `RateLimitConfig`, and `ToolConfig` creation and manipulation
- **Serialization**: Verify TOML serialization and deserialization works correctly
- **Environment Variable Expansion**: Test that `${VAR}` patterns in config values are expanded correctly
- **Provider and Tool Management**: Test adding/retrieving providers and tools from the main config
- **Security Settings**: Verify security configuration defaults and custom settings

**Key Tests**:
- `test_default_app_config()` - Validates AppConfig defaults (name, version, log level, context tokens)
- `test_hive_config_serialization()` - Ensures config can be serialized to TOML and restored
- `test_provider_config_with_rate_limit()` - Tests rate limiting configuration structure
- `test_security_config_with_tool_permissions()` - Validates tool-level permission settings

### core_types_test.rs
**33 tests** covering core type definitions (`hivecode-core::types`):

- **Message Roles**: Test `MessageRole` enum (User, Assistant, System, Tool) with Display and equality
- **Content Blocks**: Verify all `ContentBlock` variants (Text, ToolUse, ToolResult)
- **Message Creation**: Test `Message::new()`, `Message::text()`, and message composition
- **Text Extraction**: Verify `get_text()` combines text content correctly and skips non-text blocks
- **Token Counting**: Test `TokenCount` creation, addition, and serialization
- **Provider Info**: Test `ProviderInfo` structure with configuration metadata
- **Serialization**: Verify message and token serialization round-trips through JSON

**Key Tests**:
- `test_message_text()` - Creates simple text messages and verifies content
- `test_message_get_text_multiple_blocks()` - Tests combining multiple content blocks
- `test_token_count_add()` - Validates token count accumulation
- `test_message_multiple_content_types()` - Tests realistic multi-block messages with tools
- `test_provider_info_serialization()` - Ensures ProviderInfo can be serialized

### security_test.rs
**35 tests** covering the security module (`hivecode-security`):

- **Path Validation**: Test `PathValidator` for sensitive file detection and path traversal
- **Sensitive File Detection**: Verify blocking of `.env`, `.ssh/`, `.aws/`, system files, credentials, etc.
- **Permission Checking**: Test `DefaultPermissionChecker` for tool and path access control
- **Dangerous Tools**: Verify that dangerous_tool, unimplemented, and system_access are blocked by default
- **Safe Tools**: Verify safe tools (web_search, code_generator, etc.) are allowed
- **Path Safety**: Test both read and write permission checks
- **Error Handling**: Verify proper error types and messages

**Key Tests**:
- `test_path_is_sensitive_env()` - Detects .env and variants
- `test_path_is_sensitive_ssh()` - Detects SSH keys in various locations
- `test_permission_checker_blocks_dangerous_tools()` - Blocks dangerous_tool, unimplemented, system_access
- `test_permission_checker_allows_safe_tools()` - Allows legitimate tools
- `test_permission_checker_blocks_sensitive_file_paths()` - Denies access to sensitive files
- `test_permission_checker_allows_regular_file_paths()` - Allows normal file access
- `test_permission_result_check()` - Tests result type conversion

## Running the Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test core_config_test
cargo test --test core_types_test
cargo test --test security_test

# Run specific test
cargo test --test core_config_test test_default_app_config

# Run with output
cargo test -- --nocapture

# Run with verbose output
cargo test -- --nocapture --test-threads=1
```

## Test Coverage

The test suite covers:

1. **Structure Validation**: Ensuring all public types can be created and accessed correctly
2. **Defaults**: Verifying sensible defaults for all configuration structures
3. **Serialization**: Round-trip serialization through TOML and JSON
4. **Security**: Comprehensive security checks including:
   - Sensitive file detection (credentials, keys, system files)
   - Permission-based access control
   - Tool whitelisting/blacklisting
   - Path traversal prevention
5. **Error Handling**: Proper error propagation and messages

## Test Design

All tests follow Rust integration test best practices:

- Use of `#[test]` and `#[tokio::test]` attributes
- Clear, descriptive test names following `test_<function>_<scenario>()` pattern
- Isolated test functions with no dependencies between them
- Assertions using standard `assert!`, `assert_eq!`, `assert_ne!` macros
- Async tests using tokio runtime where needed
- No mock frameworks needed - tests work directly with real types

## Future Test Expansion

Potential areas for additional testing:

- Configuration file loading from disk (currently tested in-memory)
- Environment variable expansion edge cases
- Complex nested configuration structures
- Performance benchmarks for path validation
- Integration with actual file system operations
- Permission rule evaluation with complex rulesets
