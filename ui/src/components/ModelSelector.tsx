import React, { useState, useRef, useEffect } from "react";
import { useAppStore } from "@/stores/appStore";
import { ChevronDown, Zap, Eye, Cpu } from "lucide-react";
import { switchModel } from "@/lib/tauri";

export const ModelSelector: React.FC = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const dropdownRef = useRef<HTMLDivElement>(null);
  const providers = useAppStore((state) => state.providers);
  const currentModel = useAppStore((state) => state.currentModel);
  const currentProvider = useAppStore((state) => state.currentProvider);
  const switchModelStore = useAppStore((state) => state.switchModel);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleSelectModel = async (provider: string, modelId: string) => {
    switchModelStore(provider, modelId);
    await switchModel(provider, modelId);
    setIsOpen(false);
    setSearchTerm("");
  };

  const filteredProviders = providers.map((provider) => ({
    ...provider,
    models: provider.models.filter(
      (model) =>
        model.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        provider.name.toLowerCase().includes(searchTerm.toLowerCase())
    ),
  }));

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-3 py-2 rounded-lg bg-hive-bg-light dark:bg-hive-surface hover:bg-hive-border-light dark:hover:bg-hive-surface border border-hive-border-light dark:border-hive-border flex items-center justify-between transition-colors"
      >
        <div className="text-left">
          <div className="text-xs text-slate-600 dark:text-slate-400">
            Current Model
          </div>
          <div className="text-sm font-medium text-slate-900 dark:text-white">
            {currentModel}
          </div>
        </div>
        <ChevronDown
          className={`w-4 h-4 text-slate-600 dark:text-slate-400 transition-transform ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 right-0 mt-2 z-50 bg-white dark:bg-hive-surface border border-hive-border-light dark:border-hive-border rounded-lg shadow-lg">
          {/* Search Input */}
          <input
            type="text"
            placeholder="Search models..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-3 py-2 border-b border-hive-border-light dark:border-hive-border bg-hive-bg-light dark:bg-slate-800 text-sm focus:outline-none"
          />

          {/* Models List */}
          <div className="max-h-96 overflow-y-auto">
            {filteredProviders.map((provider) => (
              <div key={provider.name}>
                {provider.models.length > 0 && (
                  <>
                    {/* Provider Header */}
                    <div className="px-3 py-2 flex items-center gap-2">
                      <div
                        className={`w-2 h-2 rounded-full ${
                          provider.connected ? "bg-green-500" : "bg-gray-400"
                        }`}
                      />
                      <span className="text-xs font-semibold text-slate-600 dark:text-slate-400">
                        {provider.name}
                      </span>
                    </div>

                    {/* Models */}
                    {provider.models.map((model) => (
                      <button
                        key={model.id}
                        onClick={() =>
                          handleSelectModel(provider.name, model.id)
                        }
                        className={`w-full text-left px-6 py-3 hover:bg-hive-bg-light dark:hover:bg-hive-surface transition-colors border-b border-hive-border-light dark:border-slate-700 ${
                          currentModel === model.id
                            ? "bg-hive-cyan/10 dark:bg-hive-cyan/10 border-l-2 border-hive-cyan"
                            : ""
                        }`}
                      >
                        <div className="flex items-center justify-between">
                          <div>
                            <div className="text-sm font-medium text-slate-900 dark:text-white">
                              {model.name}
                            </div>
                            <div className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                              {Math.round(model.context_window / 1000)}K context
                            </div>
                          </div>
                          <div className="flex gap-1">
                            {model.supports_tools && (
                              <Zap className="w-4 h-4 text-hive-cyan" />
                            )}
                            {model.supports_vision && (
                              <Eye className="w-4 h-4 text-hive-cyan" />
                            )}
                          </div>
                        </div>
                      </button>
                    ))}
                  </>
                )}
              </div>
            ))}

            {filteredProviders.every((p) => p.models.length === 0) && (
              <div className="px-3 py-4 text-center text-sm text-slate-600 dark:text-slate-400">
                No models found
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
