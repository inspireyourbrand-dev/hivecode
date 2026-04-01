import React, { useState } from "react";
import { useAppStore } from "@/stores/appStore";
import { useTheme } from "@/hooks/useTheme";
import { Sun, Moon, Settings, Menu, BookMarked } from "lucide-react";
import { CostTracker } from "./CostTracker";
import { CompactButton } from "./CompactButton";
import { MemoryPanel } from "./MemoryPanel";
import { ShortcutHint } from "./KeyboardShortcuts";
import { OfflineIndicator } from "./OfflineIndicator";

export const Header: React.FC = () => {
  const [memoryPanelOpen, setMemoryPanelOpen] = useState(false);
  const currentModel = useAppStore((state) => state.currentModel);
  const currentProvider = useAppStore((state) => state.currentProvider);
  const sidebarOpen = useAppStore((state) => state.sidebarOpen);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const toggleSettings = useAppStore((state) => state.toggleSettings);
  const isOnline = useAppStore((state) => state.isOnline);
  const isDegraded = useAppStore((state) => state.isDegraded);
  const usingLocalModel = useAppStore((state) => state.usingLocalModel);
  const { theme, toggleTheme } = useTheme();

  const getModelDisplayName = (model: string) => {
    const parts = model.split("-");
    return parts
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
  };

  return (
    <>
      <header className="h-16 border-b border-hive-border-light dark:border-hive-border bg-white dark:bg-hive-surface flex items-center justify-between px-4 gap-4">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <button
            onClick={toggleSidebar}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-slate-700 rounded-lg transition-colors flex-shrink-0"
            title="Toggle sidebar (Ctrl+/)"
          >
            <Menu className="w-5 h-5 text-slate-700 dark:text-slate-300" />
          </button>

          <div className="flex items-center gap-2">
            <span className="text-xl font-bold text-slate-900 dark:text-white">
              Hive<span className="text-hive-cyan">Code</span>
            </span>
          </div>
        </div>

        {/* Center Section - Cost Tracker & Compact */}
        <div className="hidden md:flex items-center gap-3 flex-shrink-0">
          <CostTracker compact={true} />
          <CompactButton />
        </div>

        {/* Offline Indicator */}
        <div className="hidden sm:block flex-shrink-0">
          <OfflineIndicator
            isOnline={isOnline}
            isDegraded={isDegraded}
            usingLocalModel={usingLocalModel}
          />
        </div>

        {/* Right Section - Actions */}
        <div className="flex items-center gap-2 flex-shrink-0">
          {/* Model Chip */}
          <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-full bg-hive-bg-light dark:bg-hive-surface text-sm font-medium text-slate-900 dark:text-white border border-hive-border-light dark:border-hive-border" style={{ borderColor: "rgba(62, 186, 244, 0.3)" }}>
            <div className="w-2 h-2 rounded-full bg-hive-green" title="Connected" />
            <span className="text-xs text-slate-500 dark:text-slate-400">
              {currentProvider}
            </span>
            <span className="text-xs text-slate-300 dark:text-slate-600">/</span>
            <span className="text-xs font-semibold">{getModelDisplayName(currentModel)}</span>
          </div>

          {/* Memory Button */}
          <button
            onClick={() => setMemoryPanelOpen(true)}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded-lg transition-colors relative group"
            title="Memory panel (Saved memories)"
          >
            <BookMarked className="w-5 h-5 text-slate-700 dark:text-slate-300" />
            <ShortcutHint keys={["Ctrl", "M"]} label="Memory panel" />
          </button>

          {/* Theme Toggle */}
          <button
            onClick={toggleTheme}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded-lg transition-colors relative group"
            title="Toggle theme"
          >
            {theme === "dark" ? (
              <Sun className="w-5 h-5 text-yellow-500" />
            ) : (
              <Moon className="w-5 h-5 text-slate-700" />
            )}
          </button>

          {/* Settings */}
          <button
            onClick={toggleSettings}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded-lg transition-colors relative group"
            title="Settings (Ctrl+,)"
          >
            <Settings className="w-5 h-5 text-slate-700 dark:text-slate-300" />
            <ShortcutHint keys={["Ctrl", ","]} label="Settings" />
          </button>
        </div>
      </header>

      {/* Memory Panel */}
      <MemoryPanel isOpen={memoryPanelOpen} onClose={() => setMemoryPanelOpen(false)} />
    </>
  );
};
