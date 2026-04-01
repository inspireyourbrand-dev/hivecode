import { create } from "zustand";
import { ProviderInfo, ToolInfo } from "@/lib/types";

interface AppState {
  providers: ProviderInfo[];
  currentProvider: string;
  currentModel: string;
  tools: ToolInfo[];
  sidebarOpen: boolean;
  theme: "light" | "dark";
  projectPath: string | null;
  settingsOpen: boolean;
  authPanelOpen: boolean;
  permissionRequests: Array<{
    id: string;
    tool_name: string;
    input: Record<string, unknown>;
    timestamp: string;
  }>;

  // Connectivity state
  isOnline: boolean;
  isDegraded: boolean;
  usingLocalModel: boolean;

  setProviders: (providers: ProviderInfo[]) => void;
  switchModel: (provider: string, model: string) => void;
  setTools: (tools: ToolInfo[]) => void;
  toggleSidebar: () => void;
  toggleTheme: () => void;
  setProjectPath: (path: string | null) => void;
  toggleSettings: () => void;
  toggleAuthPanel: () => void;
  addPermissionRequest: (request: {
    id: string;
    tool_name: string;
    input: Record<string, unknown>;
    timestamp: string;
  }) => void;
  removePermissionRequest: (id: string) => void;
  setOnlineStatus: (isOnline: boolean, isDegraded?: boolean, usingLocalModel?: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  providers: [],
  currentProvider: "Anthropic",
  currentModel: "claude-opus-4-1",
  tools: [],
  sidebarOpen: true,
  theme: "dark",
  projectPath: null,
  settingsOpen: false,
  authPanelOpen: false,
  permissionRequests: [],
  isOnline: true,
  isDegraded: false,
  usingLocalModel: false,

  setProviders: (providers: ProviderInfo[]) => {
    set({ providers });
  },

  switchModel: (provider: string, model: string) => {
    set({
      currentProvider: provider,
      currentModel: model,
    });
  },

  setTools: (tools: ToolInfo[]) => {
    set({ tools });
  },

  toggleSidebar: () => {
    set((state) => ({
      sidebarOpen: !state.sidebarOpen,
    }));
  },

  toggleTheme: () => {
    set((state) => {
      const newTheme = state.theme === "dark" ? "light" : "dark";
      if (newTheme === "dark") {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }
      return { theme: newTheme };
    });
  },

  setProjectPath: (path: string | null) => {
    set({ projectPath: path });
  },

  toggleSettings: () => {
    set((state) => ({
      settingsOpen: !state.settingsOpen,
    }));
  },

  toggleAuthPanel: () => {
    set((state) => ({
      authPanelOpen: !state.authPanelOpen,
    }));
  },

  addPermissionRequest: (request) => {
    set((state) => ({
      permissionRequests: [...state.permissionRequests, request],
    }));
  },

  removePermissionRequest: (id: string) => {
    set((state) => ({
      permissionRequests: state.permissionRequests.filter((r) => r.id !== id),
    }));
  },

  setOnlineStatus: (isOnline: boolean, isDegraded = false, usingLocalModel = false) => {
    set({ isOnline, isDegraded, usingLocalModel });
  },
}));
