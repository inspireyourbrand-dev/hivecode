import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Message,
  ProviderInfo,
  ToolInfo,
  StreamEvent,
  PermissionRequest,
} from "./types";

const TAURI_AVAILABLE = typeof window !== "undefined" && "__TAURI__" in window;

// Mock data for development without Tauri
const mockProviders: ProviderInfo[] = [
  {
    name: "Anthropic",
    connected: true,
    models: [
      {
        id: "claude-opus-4-1",
        name: "Claude Opus 4.1",
        context_window: 200000,
        supports_tools: true,
        supports_vision: true,
      },
      {
        id: "claude-sonnet-4",
        name: "Claude Sonnet 4",
        context_window: 200000,
        supports_tools: true,
        supports_vision: true,
      },
    ],
  },
  {
    name: "OpenAI",
    connected: false,
    models: [
      {
        id: "gpt-4-turbo",
        name: "GPT-4 Turbo",
        context_window: 128000,
        supports_tools: true,
        supports_vision: true,
      },
    ],
  },
];

const mockTools: ToolInfo[] = [
  {
    name: "bash",
    description: "Execute bash commands",
    enabled: true,
  },
  {
    name: "file_operations",
    description: "Read and write files",
    enabled: true,
  },
  {
    name: "code_execution",
    description: "Execute code snippets",
    enabled: true,
  },
];

export async function sendMessage(message: string): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("send_message", { message });
  }
  // Mock response for development
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(
        `I received your message: "${message}". This is a mock response in development mode.`
      );
    }, 1000);
  });
}

export async function getConversation(): Promise<Message[]> {
  if (TAURI_AVAILABLE) {
    return invoke("get_conversation");
  }
  return [];
}

export async function listProviders(): Promise<ProviderInfo[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_providers");
  }
  return mockProviders;
}

export async function switchModel(
  provider: string,
  model: string
): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("switch_model", { provider, model });
  }
}

export async function listTools(): Promise<ToolInfo[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_tools");
  }
  return mockTools;
}

export async function getConfig(): Promise<Record<string, unknown>> {
  if (TAURI_AVAILABLE) {
    return invoke("get_config");
  }
  return {
    theme: "dark",
    defaultModel: "claude-opus-4-1",
    defaultProvider: "Anthropic",
  };
}

export async function updateConfig(
  key: string,
  value: unknown
): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("update_config", { key, value });
  }
}

export async function approvePermission(
  requestId: string,
  approved: boolean
): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("approve_permission", { requestId, approved });
  }
}

export async function openProject(path: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("open_project", { path });
  }
}

type StreamCallback = (event: StreamEvent) => void;
type PermissionCallback = (request: PermissionRequest) => void;

let streamUnlisten: (() => void) | null = null;
let toolUnlisten: (() => void) | null = null;
let permissionUnlisten: (() => void) | null = null;

export async function onStreamEvent(callback: StreamCallback): Promise<void> {
  if (TAURI_AVAILABLE) {
    streamUnlisten = await listen("stream_event", (event) => {
      callback(event.payload as StreamEvent);
    });
  }
}

export async function onToolEvent(callback: StreamCallback): Promise<void> {
  if (TAURI_AVAILABLE) {
    toolUnlisten = await listen("tool_event", (event) => {
      callback(event.payload as StreamEvent);
    });
  }
}

export async function onPermissionRequest(
  callback: PermissionCallback
): Promise<void> {
  if (TAURI_AVAILABLE) {
    permissionUnlisten = await listen("permission_request", (event) => {
      callback(event.payload as PermissionRequest);
    });
  }
}

export function unlistenAll(): void {
  if (streamUnlisten) {
    streamUnlisten();
  }
  if (toolUnlisten) {
    toolUnlisten();
  }
  if (permissionUnlisten) {
    permissionUnlisten();
  }
}

// Token and Cost Management
export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  session_cost: number;
  total_cost: number;
}

export async function getTokenUsage(): Promise<TokenUsage> {
  if (TAURI_AVAILABLE) {
    return invoke("get_token_usage");
  }
  // Mock response for development
  return {
    input_tokens: 0,
    output_tokens: 0,
    session_cost: 0,
    total_cost: 0,
  };
}

// Conversation Compaction
export interface CompactOptions {
  strategy?: "summarize" | "extract";
  keepRecent?: number;
}

export interface CompactResult {
  tokens_saved: number;
  messages_compressed: number;
  summary?: string;
}

export async function compactConversation(
  options?: CompactOptions
): Promise<CompactResult> {
  if (TAURI_AVAILABLE) {
    return invoke("compact_conversation", { options });
  }
  // Mock response for development
  return {
    tokens_saved: 2000,
    messages_compressed: 5,
    summary: "Earlier messages have been summarized",
  };
}

// Memory Management
export interface MemoryEntry {
  id: string;
  category: string;
  content: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export async function listMemories(category?: string): Promise<MemoryEntry[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_memories", { category });
  }
  // Mock response for development
  return [];
}

export async function addMemory(
  category: string,
  content: string,
  tags: string[]
): Promise<MemoryEntry> {
  if (TAURI_AVAILABLE) {
    return invoke("add_memory", { category, content, tags });
  }
  // Mock response for development
  return {
    id: `m${Date.now()}`,
    category,
    content,
    tags,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

export async function updateMemory(
  id: string,
  content: string,
  tags: string[]
): Promise<MemoryEntry> {
  if (TAURI_AVAILABLE) {
    return invoke("update_memory", { id, content, tags });
  }
  // Mock response for development
  return {
    id,
    category: "preferences",
    content,
    tags,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

export async function deleteMemory(id: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("delete_memory", { id });
  }
}

export async function searchMemories(query: string): Promise<MemoryEntry[]> {
  if (TAURI_AVAILABLE) {
    return invoke("search_memories", { query });
  }
  // Mock response for development
  return [];
}
