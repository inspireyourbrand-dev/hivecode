import React, { useState } from "react";
import { useAppStore } from "@/stores/appStore";
import { X, Plus, Trash2 } from "lucide-react";
import { updateConfig } from "@/lib/tauri";

export const SettingsPanel: React.FC = () => {
  const settingsOpen = useAppStore((state) => state.settingsOpen);
  const toggleSettings = useAppStore((state) => state.toggleSettings);
  const theme = useAppStore((state) => state.theme);
  const toggleTheme = useAppStore((state) => state.toggleTheme);
  const [activeTab, setActiveTab] = useState<
    "general" | "providers" | "permissions" | "about"
  >("general");
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({});

  const handleSaveApiKey = async (provider: string, apiKey: string) => {
    setApiKeys((prev) => ({ ...prev, [provider]: apiKey }));
    await updateConfig(`api_key_${provider}`, apiKey);
  };

  if (!settingsOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={toggleSettings}
      />

      {/* Modal */}
      <div className="fixed inset-4 md:inset-auto md:left-1/2 md:top-1/2 md:w-2xl md:h-3/4 md:transform md:-translate-x-1/2 md:-translate-y-1/2 bg-white dark:bg-hive-surface rounded-lg shadow-2xl z-50 flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-hive-border-light dark:border-hive-border">
          <h2 className="text-xl font-bold text-slate-900 dark:text-white">
            Settings
          </h2>
          <button
            onClick={toggleSettings}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-hive-border-light dark:border-hive-border px-6">
          {(
            ["general", "providers", "permissions", "about"] as const
          ).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab
                  ? "border-hive-cyan text-hive-cyan"
                  : "border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-6">
          {activeTab === "general" && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
                  Appearance
                </h3>
                <button
                  onClick={toggleTheme}
                  className={`w-full px-4 py-3 rounded-lg border border-hive-border-light dark:border-hive-border text-left transition-colors ${
                    theme === "dark"
                      ? "bg-hive-surface border-hive-cyan"
                      : "bg-white hover:bg-hive-bg-light"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-slate-900 dark:text-white">
                      Dark Mode
                    </span>
                    <div
                      className={`w-4 h-4 rounded-full transition-all ${
                        theme === "dark" ? "bg-hive-cyan" : "bg-gray-300"
                      }`}
                    />
                  </div>
                </button>
              </div>

              <div>
                <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
                  Behavior
                </h3>
                <label className="flex items-center gap-3 p-3 rounded-lg hover:bg-hive-bg-light dark:hover:bg-hive-surface cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    defaultChecked
                    className="w-4 h-4 rounded border-hive-border-light dark:border-hive-border"
                  />
                  <span className="text-sm text-slate-900 dark:text-white">
                    Auto-scroll to latest message
                  </span>
                </label>
                <label className="flex items-center gap-3 p-3 rounded-lg hover:bg-hive-bg-light dark:hover:bg-hive-surface cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    defaultChecked
                    className="w-4 h-4 rounded border-hive-border-light dark:border-hive-border"
                  />
                  <span className="text-sm text-slate-900 dark:text-white">
                    Show syntax highlighting
                  </span>
                </label>
              </div>
            </div>
          )}

          {activeTab === "providers" && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
                  API Keys
                </h3>
                <p className="text-sm text-slate-600 dark:text-slate-400 mb-4">
                  Add your API keys to enable cloud models
                </p>

                <div className="space-y-3">
                  {["Anthropic", "OpenAI", "Ollama"].map((provider) => (
                    <div key={provider}>
                      <label className="block text-sm font-medium text-slate-900 dark:text-white mb-1">
                        {provider}
                      </label>
                      <input
                        type="password"
                        placeholder={`Enter ${provider} API key`}
                        className="input-base text-sm"
                        onChange={(e) =>
                          handleSaveApiKey(provider, e.target.value)
                        }
                      />
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {activeTab === "permissions" && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
                  Tool Permissions
                </h3>
                <p className="text-sm text-slate-600 dark:text-slate-400 mb-4">
                  Control which tools require approval before execution
                </p>

                <div className="space-y-2">
                  {["bash", "file_operations", "code_execution"].map(
                    (tool) => (
                      <label
                        key={tool}
                        className="flex items-center gap-3 p-3 rounded-lg hover:bg-hive-bg-light dark:hover:bg-hive-surface cursor-pointer transition-colors"
                      >
                        <input
                          type="checkbox"
                          defaultChecked
                          className="w-4 h-4 rounded border-hive-border-light dark:border-hive-border"
                        />
                        <span className="text-sm text-slate-900 dark:text-white font-mono">
                          {tool}
                        </span>
                      </label>
                    )
                  )}
                </div>
              </div>
            </div>
          )}

          {activeTab === "about" && (
            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
                  About HiveCode
                </h3>
                <div className="space-y-3 text-sm text-slate-600 dark:text-slate-400">
                  <div>
                    <span className="font-medium text-slate-900 dark:text-white">
                      Version:
                    </span>{" "}
                    0.1.0
                  </div>
                  <div>
                    <span className="font-medium text-slate-900 dark:text-white">
                      Built with:
                    </span>{" "}
                    React 19, TypeScript, Tailwind CSS
                  </div>
                  <div className="pt-4">
                    <p>
                      HiveCode is an AI-powered coding assistant combining the
                      power of multiple language models with local execution
                      capabilities.
                    </p>
                  </div>
                  <div className="pt-4">
                    <span className="text-slate-900 dark:text-white">
                      Built by <a href="https://hivepowered.com" className="text-hive-cyan hover:text-hive-magenta transition-colors">HivePowered</a>
                    </span>
                  </div>
                  <div className="pt-2">
                    <button className="text-hive-cyan hover:text-hive-magenta transition-colors">
                      View GitHub Repository
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t border-hive-border-light dark:border-hive-border px-6 py-4 flex justify-end gap-3">
          <button
            onClick={toggleSettings}
            className="btn-secondary"
          >
            Done
          </button>
        </div>
      </div>
    </>
  );
};
