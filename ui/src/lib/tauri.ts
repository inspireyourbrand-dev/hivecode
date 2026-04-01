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

// Thinking Management
export interface ThinkingSession {
  id: string;
  thinking: string;
  tokens: number;
  timeMs: number;
  type: "reasoning" | "planning" | "analysis";
}

export async function getThinkingSession(sessionId: string): Promise<ThinkingSession | null> {
  if (TAURI_AVAILABLE) {
    return invoke("get_thinking_session", { sessionId });
  }
  return null;
}

export async function setThinkingConfig(config: Record<string, unknown>): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("set_thinking_config", { config });
  }
}

// Hooks Management
export interface Hook {
  id: string;
  name: string;
  type: "pre" | "post" | "error";
  trigger: string;
  action: string;
  enabled: boolean;
  priority?: number;
}

export async function listHooks(): Promise<Hook[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_hooks");
  }
  return [];
}

export async function createHook(hook: Omit<Hook, "id">): Promise<Hook> {
  if (TAURI_AVAILABLE) {
    return invoke("create_hook", { hook });
  }
  return { ...hook, id: `hook-${Date.now()}` };
}

export async function deleteHook(hookId: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("delete_hook", { hookId });
  }
}

export async function toggleHook(hookId: string, enabled: boolean): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("toggle_hook", { hookId, enabled });
  }
}

export interface HookLog {
  timestamp: string;
  hookId: string;
  success: boolean;
  message: string;
}

export async function getHookLog(hookId?: string, limit?: number): Promise<HookLog[]> {
  if (TAURI_AVAILABLE) {
    return invoke("get_hook_log", { hookId, limit });
  }
  return [];
}

// Branch Management
export interface ConversationBranch {
  id: string;
  name: string;
  parentId?: string;
  messageCount: number;
  cost: number;
  model: string;
  createdAt: string;
  isCurrent?: boolean;
}

export async function forkConversation(fromId: string, name?: string): Promise<ConversationBranch> {
  if (TAURI_AVAILABLE) {
    return invoke("fork_conversation", { fromId, name });
  }
  return {
    id: `branch-${Date.now()}`,
    name: name || "New Branch",
    parentId: fromId,
    messageCount: 0,
    cost: 0,
    model: "current",
    createdAt: new Date().toISOString(),
  };
}

export async function switchBranch(branchId: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("switch_branch", { branchId });
  }
}

export async function listBranches(): Promise<ConversationBranch[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_branches");
  }
  return [];
}

export async function deleteBranch(branchId: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("delete_branch", { branchId });
  }
}

export async function compareBranches(
  branchId1: string,
  branchId2: string
): Promise<{ branch1: ConversationBranch; branch2: ConversationBranch; diff: string }> {
  if (TAURI_AVAILABLE) {
    return invoke("compare_branches", { branchId1, branchId2 });
  }
  return {
    branch1: {} as ConversationBranch,
    branch2: {} as ConversationBranch,
    diff: "",
  };
}

// Offline Management
export interface OfflineStatus {
  isOnline: boolean;
  isDegraded: boolean;
  usingLocalModel: boolean;
  lastCheckTime: string;
}

export async function getOfflineStatus(): Promise<OfflineStatus> {
  if (TAURI_AVAILABLE) {
    return invoke("get_offline_status");
  }
  return {
    isOnline: true,
    isDegraded: false,
    usingLocalModel: false,
    lastCheckTime: new Date().toISOString(),
  };
}

export async function forceConnectivityCheck(): Promise<OfflineStatus> {
  if (TAURI_AVAILABLE) {
    return invoke("force_connectivity_check");
  }
  return {
    isOnline: true,
    isDegraded: false,
    usingLocalModel: false,
    lastCheckTime: new Date().toISOString(),
  };
}

export async function setOfflineConfig(config: Record<string, unknown>): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("set_offline_config", { config });
  }
}

// Project Instructions
export async function loadProjectInstructions(projectPath: string): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("load_project_instructions", { projectPath });
  }
  return "";
}

export async function saveProjectInstructions(
  projectPath: string,
  content: string
): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("save_project_instructions", { projectPath, content });
  }
}

export async function getProjectInstructionsTemplate(): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("get_project_instructions_template");
  }
  return `# HiveCode Instructions

## Instructions
Describe your project context and goals here.

## Tools
List the tools and capabilities needed:
- bash: For running shell commands
- file_operations: For reading/writing files

## Files
Specify file restrictions and patterns:
- Include: src/**, tests/**
- Exclude: node_modules/**, .git/**

## Model Preferences
Specify preferred models and settings.`;
}

// Session Replay
export interface SessionRecording {
  id: string;
  name: string;
  createdAt: string;
  events: Record<string, unknown>[];
  duration: number;
}

export async function startRecording(): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("start_recording");
  }
  return `recording-${Date.now()}`;
}

export async function stopRecording(): Promise<SessionRecording> {
  if (TAURI_AVAILABLE) {
    return invoke("stop_recording");
  }
  return {
    id: "",
    name: "",
    createdAt: new Date().toISOString(),
    events: [],
    duration: 0,
  };
}

export async function listRecordings(): Promise<SessionRecording[]> {
  if (TAURI_AVAILABLE) {
    return invoke("list_recordings");
  }
  return [];
}

export async function loadRecording(recordingId: string): Promise<SessionRecording> {
  if (TAURI_AVAILABLE) {
    return invoke("load_recording", { recordingId });
  }
  return {
    id: recordingId,
    name: "",
    createdAt: new Date().toISOString(),
    events: [],
    duration: 0,
  };
}

export async function deleteRecording(recordingId: string): Promise<void> {
  if (TAURI_AVAILABLE) {
    return invoke("delete_recording", { recordingId });
  }
}

export async function exportRecording(
  recordingId: string,
  format: "markdown" | "json"
): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("export_recording", { recordingId, format });
  }
  return "";
}

// Cost Optimizer
export interface CostAnalysis {
  sessionCost: number;
  breakdown: Array<{
    model: string;
    cost: number;
    percentage: number;
    tokenCount: number;
  }>;
  recommendations: Array<{
    id: string;
    title: string;
    description: string;
    savings: number;
    difficulty: "Easy" | "Medium" | "Advanced";
    action: string;
  }>;
}

export async function getCostAnalysis(): Promise<CostAnalysis> {
  if (TAURI_AVAILABLE) {
    return invoke("get_cost_analysis");
  }
  return {
    sessionCost: 0,
    breakdown: [],
    recommendations: [],
  };
}

export async function getCostBreakdown(): Promise<Array<{
  model: string;
  cost: number;
  percentage: number;
  tokenCount: number;
}>> {
  if (TAURI_AVAILABLE) {
    return invoke("get_cost_breakdown");
  }
  return [];
}

export async function getDailyCostTrend(): Promise<Array<{
  date: string;
  cost: number;
}>> {
  if (TAURI_AVAILABLE) {
    return invoke("get_daily_cost_trend");
  }
  return [];
}

// Diff Capture
export interface FileDiff {
  filename: string;
  additions: number;
  deletions: number;
  hunks: Array<{
    id: string;
    oldStart: number;
    newStart: number;
    oldLines: number;
    newLines: number;
    lines: Array<{
      type: "add" | "remove" | "context";
      content: string;
      lineNumber?: number;
    }>;
  }>;
}

export async function captureFileBefore(filePath: string): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("capture_file_before", { filePath });
  }
  return "";
}

export async function captureFileAfter(filePath: string): Promise<string> {
  if (TAURI_AVAILABLE) {
    return invoke("capture_file_after", { filePath });
  }
  return "";
}

export async function getPendingDiffs(): Promise<FileDiff[]> {
  if (TAURI_AVAILABLE) {
    return invoke("get_pending_diffs");
  }
  return [];
}
