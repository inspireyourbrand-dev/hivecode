import React, { useEffect } from "react";
import { useAppStore } from "@/stores/appStore";
import { listProviders, listTools } from "@/lib/tauri";
import { ModelSelector } from "./ModelSelector";
import { ChevronDown, FolderOpen, Zap, X, MessageSquarePlus, History, Cloud } from "lucide-react";
import { useState } from "react";

export const Sidebar: React.FC = () => {
  const sidebarOpen = useAppStore((state) => state.sidebarOpen);
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const providers = useAppStore((state) => state.providers);
  const setProviders = useAppStore((state) => state.setProviders);
  const tools = useAppStore((state) => state.tools);
  const setTools = useAppStore((state) => state.setTools);
  const projectPath = useAppStore((state) => state.projectPath);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(["models", "tools"])
  );

  useEffect(() => {
    const loadData = async () => {
      try {
        const loadedProviders = await listProviders();
        setProviders(loadedProviders);

        const loadedTools = await listTools();
        setTools(loadedTools);
      } catch (error) {
        console.error("Failed to load providers and tools:", error);
      }
    };

    loadData();
  }, [setProviders, setTools]);

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const sidebarContent = (
    <div className="flex flex-col h-full">
      {/* Logo */}
      <div className="px-4 py-4 border-b border-hive-border-light dark:border-hive-border">
        <div className="flex items-center gap-3">
          <svg width="28" height="28" viewBox="0 0 80 80" className="flex-shrink-0" style={{ filter: "drop-shadow(0 0 6px rgba(62, 186, 244, 0.5))" }}>
            <defs>
              <linearGradient id="sidebarHexGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#3ebaf4" />
                <stop offset="100%" stopColor="#df30ff" />
              </linearGradient>
            </defs>
            <path d="M40 10 L65 22.5 L65 57.5 L40 70 L15 57.5 L15 22.5 Z" fill="none" stroke="url(#sidebarHexGrad)" strokeWidth="3" />
            <line x1="28" y1="28" x2="28" y2="52" stroke="url(#sidebarHexGrad)" strokeWidth="3" strokeLinecap="round" />
            <line x1="52" y1="28" x2="52" y2="52" stroke="url(#sidebarHexGrad)" strokeWidth="3" strokeLinecap="round" />
            <line x1="28" y1="40" x2="52" y2="40" stroke="url(#sidebarHexGrad)" strokeWidth="3" strokeLinecap="round" />
          </svg>
          <span className="font-bold text-lg text-slate-900 dark:text-white">
            Hive<span className="text-hive-cyan">Code</span>
          </span>
        </div>
      </div>

      {/* Scrollable Content */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-6">
        {/* Project Explorer */}
        <div>
          <button
            onClick={() => toggleSection("project")}
            className="w-full flex items-center justify-between px-2 py-2 text-sm font-semibold text-slate-900 dark:text-white hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <div className="flex items-center gap-2">
              <FolderOpen className="w-4 h-4" />
              Project
            </div>
            <ChevronDown
              className={`w-4 h-4 transition-transform ${
                expandedSections.has("project") ? "rotate-180" : ""
              }`}
            />
          </button>
          {expandedSections.has("project") && (
            <div className="mt-2 pl-4 text-sm text-slate-600 dark:text-slate-400">
              {projectPath ? (
                <div className="p-2 bg-hive-bg-light dark:bg-slate-700 rounded">
                  <div className="text-xs font-mono truncate">{projectPath}</div>
                </div>
              ) : (
                <p>No project loaded. Open a project from settings.</p>
              )}
            </div>
          )}
        </div>

        {/* Model Selector */}
        <div>
          <button
            onClick={() => toggleSection("models")}
            className="w-full flex items-center justify-between px-2 py-2 text-sm font-semibold text-slate-900 dark:text-white hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4" />
              Models
            </div>
            <ChevronDown
              className={`w-4 h-4 transition-transform ${
                expandedSections.has("models") ? "rotate-180" : ""
              }`}
            />
          </button>
          {expandedSections.has("models") && (
            <div className="mt-2">
              <ModelSelector />
            </div>
          )}
        </div>

        {/* Tools */}
        <div>
          <button
            onClick={() => toggleSection("tools")}
            className="w-full flex items-center justify-between px-2 py-2 text-sm font-semibold text-slate-900 dark:text-white hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4" />
              Tools
            </div>
            <ChevronDown
              className={`w-4 h-4 transition-transform ${
                expandedSections.has("tools") ? "rotate-180" : ""
              }`}
            />
          </button>
          {expandedSections.has("tools") && (
            <div className="mt-2 space-y-2">
              {tools.map((tool) => (
                <div
                  key={tool.name}
                  className="p-2 rounded bg-hive-bg-light dark:bg-slate-700 text-sm"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-slate-900 dark:text-white">
                      {tool.name}
                    </span>
                    <div
                      className={`w-2 h-2 rounded-full ${
                        tool.enabled ? "bg-hive-green" : "bg-gray-400"
                      }`}
                    />
                  </div>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                    {tool.description}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Provider Status */}
        <div>
          <button
            onClick={() => toggleSection("providers")}
            className="w-full flex items-center justify-between px-2 py-2 text-sm font-semibold text-slate-900 dark:text-white hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <div className="flex items-center gap-2">
              <Cloud className="w-4 h-4" />
              Providers
            </div>
            <ChevronDown
              className={`w-4 h-4 transition-transform ${
                expandedSections.has("providers") ? "rotate-180" : ""
              }`}
            />
          </button>
          {expandedSections.has("providers") && (
            <div className="mt-2 space-y-2">
              {providers.map((provider) => (
                <div
                  key={provider.name}
                  className="p-2 rounded bg-hive-bg-light dark:bg-slate-700 text-sm"
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-slate-900 dark:text-white">
                      {provider.name}
                    </span>
                    <div
                      className={`w-2 h-2 rounded-full ${
                        provider.connected ? "bg-green-500" : "bg-gray-400"
                      }`}
                      title={
                        provider.connected ? "Connected" : "Disconnected"
                      }
                    />
                  </div>
                  <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                    {provider.models.length} model{provider.models.length !== 1 ? "s" : ""}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="px-4 py-4 border-t border-hive-border-light dark:border-hive-border space-y-2">
        <button className="w-full btn-primary text-sm flex items-center justify-center gap-2">
          <MessageSquarePlus className="w-4 h-4" />
          New Chat
        </button>
        <div className="text-[10px] text-center text-slate-500 dark:text-slate-600">
          v0.1.0 &middot; HivePowered
        </div>
      </div>
    </div>
  );

  return (
    <>
      {/* Desktop Sidebar */}
      <div
        className={`hidden lg:flex flex-col w-64 bg-hive-bg-light dark:bg-hive-surface border-r border-hive-border-light dark:border-hive-border transition-transform ${
          sidebarOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        {sidebarContent}
      </div>

      {/* Mobile Sidebar */}
      {sidebarOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 bg-black/50 lg:hidden z-40"
            onClick={toggleSidebar}
          />

          {/* Mobile Drawer */}
          <div className="fixed top-0 left-0 h-screen w-64 bg-hive-bg-light dark:bg-hive-surface border-r border-hive-border-light dark:border-hive-border z-50 flex flex-col lg:hidden">
            <div className="absolute top-4 right-4">
              <button
                onClick={toggleSidebar}
                className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            {sidebarContent}
          </div>
        </>
      )}
    </>
  );
};
