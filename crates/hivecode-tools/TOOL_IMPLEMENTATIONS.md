# HiveCode Tool Implementations

This document describes the 7 new tool implementations added to the HiveCode tool system.

## Overview

All tools implement the `Tool` trait from `traits.rs` and follow the same patterns as existing tools:
- Async execution via `#[async_trait]`
- JSON schema-based input validation
- Proper error handling with `ToolError` types
- Permission checking via `ToolContext`
- Comprehensive test coverage

## New Tools

### 1. NotebookEditTool (`notebook_edit.rs`)

**Purpose**: Edit and manage Jupyter notebook (.ipynb) files

**Capabilities**:
- Read notebook structure and list all cells
- Edit cell content by index (code, markdown, raw types)
- Insert new cells at specified positions
- Delete cells by index
- Validate notebook JSON structure

**Input Actions**:
- `read`: Returns all cells with summaries
- `edit`: Modify cell source code
- `insert`: Add new cell at index
- `delete`: Remove cell at index

**Example Usage**:
```json
{
  "action": "edit",
  "file_path": "/path/to/notebook.ipynb",
  "cell_index": 0,
  "new_source": "print('updated code')"
}
```

**Key Features**:
- Parses and validates notebook JSON structure
- Supports all three notebook cell types
- Preserves notebook metadata during edits
- Returns cell previews on read action

---

### 2. TodoTool (`todo_tool.rs`)

**Purpose**: In-session task and todo list management

**Capabilities**:
- Create new todo items with unique IDs and status
- Update todo content and status (pending, in_progress, completed)
- Delete todo items
- List all todos with formatted output
- Clear all todos at once

**Input Actions**:
- `create`: Create a new todo item
- `update`: Modify existing todo
- `delete`: Remove todo by ID
- `list`: Display all todos
- `clear`: Remove all todos

**Example Usage**:
```json
{
  "action": "create",
  "id": "task-123",
  "content": "Implement feature X",
  "status": "pending"
}
```

**Key Features**:
- Thread-safe storage via `Arc<Mutex<HashMap>>`
- Timestamps on creation (using chrono)
- Validates status values
- Formatted output for CLI display
- Global storage accessible across tool calls

---

### 3. ToolSearchTool (`tool_search.rs`)

**Purpose**: Discover and search registered tools in the system

**Capabilities**:
- Fuzzy match tool names and descriptions
- List all available tools with metadata
- Get detailed information about specific tools
- Search for tools by query with match scores

**Input Actions**:
- `search`: Find matching tools by query
- `list`: List all registered tools
- `details`: Get info about a specific tool

**Example Usage**:
```json
{
  "action": "search",
  "query": "file"
}
```

**Key Features**:
- Case-insensitive fuzzy matching
- Supports partial character matching
- Returns match scores for results
- Lists enabled status for each tool
- Includes tool schemas in details

---

### 4. ConfigTool (`config_tool.rs`)

**Purpose**: Manage system configuration settings

**Capabilities**:
- Read configuration values
- Set configuration with validation
- List all current settings
- Reset individual configs to defaults
- Reset all configs to defaults

**Input Actions**:
- `get`: Read a config value
- `set`: Update a config value with validation
- `list`: Display all configuration
- `reset`: Reset one config to default
- `reset_all`: Reset all configs to defaults

**Example Usage**:
```json
{
  "action": "set",
  "key": "timeout",
  "value": 60
}
```

**Default Configuration**:
- `timeout`: 120 (seconds)
- `max_retries`: 3
- `verbose`: false
- `output_format`: "text" (json, csv options)

**Key Features**:
- Type validation for known keys
- Custom key support with flexible values
- Immutable defaults for standard keys
- Formatted list output
- Thread-safe storage

---

### 5. DiffTool (`diff_tool.rs`)

**Purpose**: Generate unified and inline diffs between files and strings

**Capabilities**:
- Generate unified diff format between files
- Generate unified diff format between strings
- Generate inline line-by-line diffs
- Show context lines around changes
- Identify identical vs. different content

**Input Actions**:
- `file_diff`: Compare two files
- `string_diff`: Compare two text strings
- `inline_diff`: Show line-by-line differences

**Example Usage**:
```json
{
  "action": "string_diff",
  "text_1": "hello\nworld",
  "text_2": "hello\nthere",
  "context_lines": 3
}
```

**Key Features**:
- Unified diff format with headers
- Inline diffs with OLD/NEW labels
- Line-by-line comparison
- Permission-checked file access
- Returns metadata about differences

---

### 6. GitTool (`git_tool.rs`)

**Purpose**: Execute Git operations for version control

**Capabilities**:
- Check repository status (porcelain format)
- Show diffs for files or whole repo
- View commit history (log)
- Blame specific files
- Stage files (git add)
- Create commits with messages
- List branches or create new ones
- Show commit content (git show)

**Input Actions**:
- `status`: Show changed files
- `diff`: Show file changes
- `log`: Show commit history
- `blame`: Show blame information
- `add`: Stage file for commit
- `commit`: Create commit
- `branch`: List or create branches
- `show`: Show commit content

**Example Usage**:
```json
{
  "action": "commit",
  "message": "Fix bug in feature X"
}
```

**Key Features**:
- Works in any Git repository
- Proper error reporting
- Returns metadata (commit counts, file changes)
- Supports file-specific operations
- Uses porcelain format for parsing

---

### 7. LspTool (`lsp_tool.rs`)

**Purpose**: Language Server Protocol (LSP) integration for code intelligence

**Capabilities**:
- Get hover information at cursor position
- Request code completions
- Get diagnostic information
- Find symbol definitions
- Find all references to a symbol

**Input Actions**:
- `hover`: Get type/symbol info at position
- `completions`: Get code suggestions
- `diagnostics`: Get errors/warnings
- `definition`: Find symbol definition
- `references`: Find all references

**Example Usage**:
```json
{
  "action": "hover",
  "file_path": "/path/to/file.rs",
  "position": {
    "line": 10,
    "character": 5
  }
}
```

**Position Format**:
- Line: 0-indexed line number
- Character: 0-indexed character position in line

**Key Features**:
- LSP-standard position protocol
- Returns range information
- Supports multiple symbol operations
- Diagnostic severity levels
- Completion item metadata

---

## Integration Points

### Registration in Registry

All tools are registered in `create_default_registry()` in `lib.rs`:

```rust
registry.register(Arc::new(notebook_edit::NotebookEditTool::new()));
registry.register(Arc::new(todo_tool::TodoTool::new()));
registry.register(Arc::new(tool_search::ToolSearchTool::new()));
registry.register(Arc::new(config_tool::ConfigTool::new()));
registry.register(Arc::new(diff_tool::DiffTool::new()));
registry.register(Arc::new(git_tool::GitTool::new()));
registry.register(Arc::new(lsp_tool::LspTool::new()));
```

### Module Declarations

All tools are declared as public modules in `lib.rs`:

```rust
pub mod notebook_edit;
pub mod todo_tool;
pub mod tool_search;
pub mod config_tool;
pub mod diff_tool;
pub mod git_tool;
pub mod lsp_tool;
```

## Implementation Notes

### Dependencies

- All tools use only existing workspace dependencies
- Added `chrono` import to `Cargo.toml` (already in workspace)
- No new external crates required

### Error Handling

Tools use consistent error handling:
- `ToolError::InvalidInput` for validation failures
- `ToolError::FileNotFound` for missing files
- `ToolError::ExecutionFailed` for runtime errors
- `ToolError::PermissionDenied` for security violations

### Permission Checking

All tools that access files use:
```rust
ctx.permission_checker
    .check_path(&path, is_write)
    .await
    .check()?;
```

### Testing

Each tool includes comprehensive unit tests covering:
- Valid input processing
- Error cases
- Edge conditions
- Schema validation

## Design Patterns

### Tool Struct Pattern

```rust
pub struct ToolName;

impl ToolName {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolName {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ToolName {
    // Implementation
}
```

### Input Schema Pattern

Each tool defines a complete JSON schema:

```rust
fn input_schema(&self) -> Value {
    json!({
        "type": "object",
        "properties": { /* ... */ },
        "required": ["field"]
    })
}
```

### Result Pattern

```rust
Ok(ToolResult::success_with_metadata(
    content: impl Into<String>,
    metadata: serde_json::Value
))
```

## Future Enhancements

Potential improvements for future versions:

1. **NotebookEditTool**: Add cell output capture, execution support
2. **TodoTool**: Persist todos to file/database, add filtering
3. **ToolSearchTool**: Full registry integration with schema matching
4. **ConfigTool**: Configuration file persistence (TOML/JSON)
5. **DiffTool**: Three-way diffs, patch application
6. **GitTool**: Advanced operations (rebase, merge, cherry-pick)
7. **LspTool**: Real LSP server connection, protocol implementation

## Files Modified

- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/lib.rs`
  - Added module declarations
  - Updated `create_default_registry()` function

- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/Cargo.toml`
  - Added `chrono` workspace dependency

## Files Created

- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/notebook_edit.rs` (13 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/todo_tool.rs` (9.7 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/tool_search.rs` (8.5 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/config_tool.rs` (12 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/diff_tool.rs` (10 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/git_tool.rs` (8.4 KB)
- `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/lsp_tool.rs` (12 KB)

**Total: 73.6 KB of new tool implementations**
