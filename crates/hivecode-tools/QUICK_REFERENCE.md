# HiveCode Tools - Quick Reference

## Tool Summary Matrix

| Tool | Location | Purpose | File Size |
|------|----------|---------|-----------|
| NotebookEditTool | `notebook_edit.rs` | Edit Jupyter notebooks | 13 KB |
| TodoTool | `todo_tool.rs` | Manage task lists | 9.7 KB |
| ToolSearchTool | `tool_search.rs` | Discover tools | 8.5 KB |
| ConfigTool | `config_tool.rs` | Manage settings | 12 KB |
| DiffTool | `diff_tool.rs` | Generate diffs | 10 KB |
| GitTool | `git_tool.rs` | Git operations | 8.4 KB |
| LspTool | `lsp_tool.rs` | Code intelligence | 12 KB |

## Quick API Examples

### NotebookEditTool

```json
// Read cells
{"action": "read", "file_path": "notebook.ipynb"}

// Edit a cell
{"action": "edit", "file_path": "notebook.ipynb", "cell_index": 0, "new_source": "print('x')"}

// Insert cell
{"action": "insert", "file_path": "notebook.ipynb", "cell_type": "code", "new_source": "x = 1"}

// Delete cell
{"action": "delete", "file_path": "notebook.ipynb", "cell_index": 0}
```

### TodoTool

```json
// Create todo
{"action": "create", "id": "t1", "content": "Task", "status": "pending"}

// Update todo
{"action": "update", "id": "t1", "status": "completed"}

// List todos
{"action": "list"}

// Delete todo
{"action": "delete", "id": "t1"}
```

### ToolSearchTool

```json
// Search tools
{"action": "search", "query": "file"}

// List all tools
{"action": "list"}

// Get tool details
{"action": "details", "tool_name": "bash"}
```

### ConfigTool

```json
// Get value
{"action": "get", "key": "timeout"}

// Set value
{"action": "set", "key": "timeout", "value": 60}

// List all
{"action": "list"}

// Reset one
{"action": "reset", "key": "timeout"}

// Reset all
{"action": "reset_all"}
```

### DiffTool

```json
// Compare files
{
  "action": "file_diff",
  "file_path_1": "old.txt",
  "file_path_2": "new.txt"
}

// Compare strings
{
  "action": "string_diff",
  "text_1": "old text",
  "text_2": "new text"
}

// Inline diff
{
  "action": "inline_diff",
  "text_1": "line 1\nline 2",
  "text_2": "line 1\nmodified"
}
```

### GitTool

```json
// Status
{"action": "status"}

// Log
{"action": "log", "lines": 10}

// Diff
{"action": "diff", "file_path": "src/main.rs"}

// Add file
{"action": "add", "file_path": "file.txt"}

// Commit
{"action": "commit", "message": "Fix bug"}

// Branch
{"action": "branch"}

// Blame
{"action": "blame", "file_path": "src/main.rs"}
```

### LspTool

```json
// Hover
{
  "action": "hover",
  "file_path": "main.rs",
  "position": {"line": 10, "character": 5}
}

// Completions
{
  "action": "completions",
  "file_path": "main.rs",
  "position": {"line": 10, "character": 5}
}

// Diagnostics
{"action": "diagnostics", "file_path": "main.rs"}

// Definition
{
  "action": "definition",
  "file_path": "main.rs",
  "position": {"line": 10, "character": 5}
}

// References
{
  "action": "references",
  "file_path": "main.rs",
  "position": {"line": 10, "character": 5}
}
```

## Status Constants

### TodoTool Statuses
- `"pending"` - Not started
- `"in_progress"` - Currently working on it
- `"completed"` - Finished

### ConfigTool Defaults
- `timeout`: 120
- `max_retries`: 3
- `verbose`: false
- `output_format`: "text"

## Error Handling

All tools return one of:
- `ToolResult::success(message)` - Success with message
- `ToolResult::success_with_metadata(message, metadata)` - Success with extra data
- `ToolError::InvalidInput(msg)` - Bad input
- `ToolError::FileNotFound(path)` - File missing
- `ToolError::ExecutionFailed(msg)` - Execution error
- `ToolError::PermissionDenied(msg)` - Access denied

## Integration Checklist

- [x] All 7 tools implemented
- [x] Tools registered in `create_default_registry()`
- [x] Modules declared in `lib.rs`
- [x] Dependencies added to `Cargo.toml`
- [x] Comprehensive tests included
- [x] Error handling implemented
- [x] Permission checking enabled
- [x] JSON schemas defined
- [x] Documentation created

## File Locations

```
/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/src/
├── notebook_edit.rs
├── todo_tool.rs
├── tool_search.rs
├── config_tool.rs
├── diff_tool.rs
├── git_tool.rs
├── lsp_tool.rs
└── lib.rs (updated)
```

Cargo.toml: `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/Cargo.toml`

Documentation: `/sessions/sharp-nice-allen/mnt/HiveCode/hivecode/crates/hivecode-tools/TOOL_IMPLEMENTATIONS.md`
