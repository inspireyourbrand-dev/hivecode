export type Role = "user" | "assistant" | "system" | "tool";
export type ContentBlockType = "text" | "tool_use" | "tool_result";

export interface ToolUse {
  id: string;
  name: string;
  input: Record<string, unknown>;
}

export interface ToolResult {
  tool_use_id: string;
  content: string;
  is_error: boolean;
}

export interface ContentBlock {
  type: ContentBlockType;
  text?: string;
  tool_use?: ToolUse;
  tool_result?: ToolResult;
}

export interface Message {
  id: string;
  role: Role;
  content: ContentBlock[];
  timestamp: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  context_window: number;
  supports_tools: boolean;
  supports_vision: boolean;
}

export interface ProviderInfo {
  name: string;
  models: ModelInfo[];
  connected: boolean;
}

export interface ToolInfo {
  name: string;
  description: string;
  enabled: boolean;
}

export interface StreamEvent {
  type:
    | "content_delta"
    | "tool_use_start"
    | "tool_input_delta"
    | "tool_use_end"
    | "usage"
    | "message_end"
    | "error";
  data: unknown;
}

export interface PermissionRequest {
  id: string;
  tool_name: string;
  input: Record<string, unknown>;
  timestamp: string;
}

export interface UsageInfo {
  input_tokens: number;
  output_tokens: number;
  cache_creation_input_tokens?: number;
  cache_read_input_tokens?: number;
}
