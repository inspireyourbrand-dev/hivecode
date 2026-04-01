import { create } from "zustand";
import { Message, ContentBlock } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";

export interface SessionSummary {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  model_used: string;
  token_count: number;
  message_count: number;
}

interface ChatState {
  messages: Message[];
  isStreaming: boolean;
  currentStreamContent: string;
  currentToolUseId: string | null;
  currentSessionId: string | null;
  sessions: SessionSummary[];
  sendMessage: (content: string) => void;
  appendStreamDelta: (delta: string) => void;
  startToolUse: (toolUseId: string, name: string) => void;
  appendToolInputDelta: (delta: string) => void;
  completeToolUse: () => void;
  completeStream: () => void;
  clearConversation: () => void;
  addMessage: (message: Message) => void;
  setMessages: (messages: Message[]) => void;
  setIsStreaming: (isStreaming: boolean) => void;
  // Session management
  setCurrentSessionId: (id: string | null) => void;
  setSessions: (sessions: SessionSummary[]) => void;
  loadSession: (sessionId: string) => Promise<void>;
  newSession: (model?: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<void>;
  saveCurrentSession: (title?: string) => Promise<void>;
  refreshSessions: () => Promise<void>;
  searchSessions: (query: string) => Promise<SessionSummary[]>;
  autoSaveSession: () => Promise<void>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  isStreaming: false,
  currentStreamContent: "",
  currentToolUseId: null,
  currentSessionId: null,
  sessions: [],

  sendMessage: (content: string) => {
    const userMessage: Message = {
      id: `msg-${Date.now()}`,
      role: "user",
      content: [{ type: "text", text: content }],
      timestamp: new Date().toISOString(),
    };

    set((state) => ({
      messages: [...state.messages, userMessage],
      isStreaming: true,
      currentStreamContent: "",
    }));

    // Auto-save after sending message
    get().autoSaveSession();
  },

  appendStreamDelta: (delta: string) => {
    set((state) => ({
      currentStreamContent: state.currentStreamContent + delta,
    }));
  },

  startToolUse: (toolUseId: string, name: string) => {
    set({ currentToolUseId: toolUseId });
  },

  appendToolInputDelta: (delta: string) => {
    // Tool input is being streamed
  },

  completeToolUse: () => {
    set({ currentToolUseId: null });
  },

  completeStream: () => {
    const state = get();
    if (state.currentStreamContent || state.currentToolUseId) {
      const contentBlocks: ContentBlock[] = [];

      if (state.currentStreamContent) {
        contentBlocks.push({
          type: "text",
          text: state.currentStreamContent,
        });
      }

      const assistantMessage: Message = {
        id: `msg-${Date.now()}`,
        role: "assistant",
        content: contentBlocks,
        timestamp: new Date().toISOString(),
      };

      set((state) => ({
        messages: [...state.messages, assistantMessage],
        isStreaming: false,
        currentStreamContent: "",
        currentToolUseId: null,
      }));

      // Auto-save after receiving response
      get().autoSaveSession();
    }
  },

  clearConversation: () => {
    set({
      messages: [],
      isStreaming: false,
      currentStreamContent: "",
      currentToolUseId: null,
      currentSessionId: null,
    });
  },

  addMessage: (message: Message) => {
    set((state) => ({
      messages: [...state.messages, message],
    }));
  },

  setMessages: (messages: Message[]) => {
    set({ messages });
  },

  setIsStreaming: (isStreaming: boolean) => {
    set({ isStreaming });
  },

  setCurrentSessionId: (id: string | null) => {
    set({ currentSessionId: id });
  },

  setSessions: (sessions: SessionSummary[]) => {
    set({ sessions });
  },

  loadSession: async (sessionId: string) => {
    try {
      const session = await invoke("load_session", { sessionId });
      const messages = (session as any).messages.map((msg: any) => ({
        id: msg.id,
        role: msg.role as "user" | "assistant" | "system" | "tool",
        content: [{ type: "text", text: msg.content }],
        timestamp: msg.timestamp,
      }));

      set({
        messages,
        currentSessionId: sessionId,
        isStreaming: false,
        currentStreamContent: "",
        currentToolUseId: null,
      });
    } catch (error) {
      console.error("Failed to load session:", error);
    }
  },

  newSession: async (model?: string) => {
    try {
      const session = await invoke("new_session", { model });
      const sessionId = (session as any).id;

      set({
        messages: [],
        currentSessionId: sessionId,
        isStreaming: false,
        currentStreamContent: "",
        currentToolUseId: null,
      });

      await get().refreshSessions();
    } catch (error) {
      console.error("Failed to create new session:", error);
    }
  },

  deleteSession: async (sessionId: string) => {
    try {
      await invoke("delete_session", { sessionId });

      const state = get();
      if (state.currentSessionId === sessionId) {
        set({
          messages: [],
          currentSessionId: null,
        });
      }

      await get().refreshSessions();
    } catch (error) {
      console.error("Failed to delete session:", error);
    }
  },

  saveCurrentSession: async (title?: string) => {
    try {
      const state = get();
      await invoke("save_current_conversation", {
        title: title || undefined,
      });

      await get().refreshSessions();
    } catch (error) {
      console.error("Failed to save session:", error);
    }
  },

  refreshSessions: async () => {
    try {
      const sessions = (await invoke("list_sessions")) as SessionSummary[];
      set({ sessions });
    } catch (error) {
      console.error("Failed to refresh sessions:", error);
    }
  },

  searchSessions: async (query: string) => {
    try {
      const results = (await invoke("search_sessions", {
        query,
      })) as SessionSummary[];
      return results;
    } catch (error) {
      console.error("Failed to search sessions:", error);
      return [];
    }
  },

  autoSaveSession: async () => {
    try {
      const state = get();
      // Save without explicit title - will auto-generate from first message
      await invoke("save_current_conversation", {});

      // Refresh sessions list
      await get().refreshSessions();
    } catch (error) {
      // Silently fail for auto-save - user can manually save if needed
      console.debug("Auto-save skipped:", error);
    }
  },
}));
