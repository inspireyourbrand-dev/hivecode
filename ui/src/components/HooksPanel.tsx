import React, { useState } from "react";
import { Zap, Plus, Trash2, Copy, Eye, EyeOff } from "lucide-react";

interface Hook {
  id: string;
  name: string;
  type: "pre" | "post" | "error";
  trigger: string;
  action: string;
  enabled: boolean;
  priority?: number;
}

interface HookLog {
  timestamp: string;
  hookId: string;
  success: boolean;
  message: string;
}

interface HooksPanelProps {
  hooks: Hook[];
  logs: HookLog[];
  onToggleHook?: (hookId: string, enabled: boolean) => void;
  onCreateHook?: (hook: Omit<Hook, "id">) => void;
  onDeleteHook?: (hookId: string) => void;
  onReorderHooks?: (hookIds: string[]) => void;
}

const HOOK_TEMPLATES = {
  "Auto-lint on save": {
    type: "post" as const,
    trigger: "file_saved",
    action: "run_linter",
  },
  "Log all bash commands": {
    type: "pre" as const,
    trigger: "bash_command",
    action: "log_to_memory",
  },
  "Auto-format code": {
    type: "post" as const,
    trigger: "code_generated",
    action: "format_code",
  },
  "Check permissions": {
    type: "pre" as const,
    trigger: "any_operation",
    action: "check_permissions",
  },
};

export const HooksPanel: React.FC<HooksPanelProps> = ({
  hooks,
  logs,
  onToggleHook,
  onCreateHook,
  onDeleteHook,
  onReorderHooks,
}) => {
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [showLogs, setShowLogs] = useState(false);
  const [newHook, setNewHook] = useState({
    name: "",
    type: "post" as const,
    trigger: "",
    action: "",
  });
  const [draggedHookId, setDraggedHookId] = useState<string | null>(null);

  const handleCreateHook = () => {
    if (newHook.name && newHook.trigger && newHook.action) {
      onCreateHook?.({
        ...newHook,
        enabled: true,
        priority: hooks.length + 1,
      });
      setNewHook({ name: "", type: "post", trigger: "", action: "" });
      setShowCreateDialog(false);
    }
  };

  const handleDragStart = (hookId: string) => {
    setDraggedHookId(hookId);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDrop = (hookId: string) => {
    if (draggedHookId && draggedHookId !== hookId) {
      const newOrder = hooks
        .map((h) => h.id)
        .map((id) => (id === draggedHookId ? hookId : id === hookId ? draggedHookId : id));
      onReorderHooks?.(newOrder);
    }
    setDraggedHookId(null);
  };

  const recentLogs = logs.slice(-10);

  return (
    <div className="h-full flex flex-col rounded-lg border border-hive-border bg-hive-surface">
      {/* Header */}
      <div className="px-4 py-3 border-b border-hive-border/50 flex items-center justify-between flex-shrink-0">
        <h3 className="text-sm font-semibold text-white flex items-center gap-2">
          <Zap className="w-4 h-4 text-hive-yellow" />
          Hooks
        </h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowLogs(!showLogs)}
            className="p-1.5 rounded hover:bg-hive-border transition-colors text-slate-400 hover:text-hive-cyan"
            title="Toggle logs"
          >
            {showLogs ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
          </button>
          <button
            onClick={() => setShowCreateDialog(true)}
            className="p-1.5 rounded hover:bg-hive-border transition-colors text-hive-cyan"
            title="New hook"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Create Hook Dialog */}
      {showCreateDialog && (
        <div className="absolute inset-0 bg-black/50 flex items-center justify-center z-50 rounded-lg">
          <div className="bg-hive-surface border border-hive-cyan p-4 rounded-lg max-w-sm w-full mx-4 max-h-96 overflow-y-auto">
            <h4 className="text-sm font-semibold text-white mb-3">Create Hook</h4>

            <div className="space-y-3 mb-4">
              <div>
                <label className="text-xs text-slate-400 block mb-1">Hook Name</label>
                <input
                  type="text"
                  value={newHook.name}
                  onChange={(e) => setNewHook({ ...newHook, name: e.target.value })}
                  placeholder="e.g., Auto-lint on save"
                  className="w-full px-3 py-2 rounded bg-hive-bg border border-hive-border text-white text-sm focus:border-hive-cyan focus:outline-none"
                />
              </div>

              <div>
                <label className="text-xs text-slate-400 block mb-1">Hook Type</label>
                <select
                  value={newHook.type}
                  onChange={(e) =>
                    setNewHook({
                      ...newHook,
                      type: e.target.value as "pre" | "post" | "error",
                    })
                  }
                  className="w-full px-3 py-2 rounded bg-hive-bg border border-hive-border text-white text-sm focus:border-hive-cyan focus:outline-none"
                >
                  <option value="pre">Pre (before operation)</option>
                  <option value="post">Post (after operation)</option>
                  <option value="error">Error (on failure)</option>
                </select>
              </div>

              <div>
                <label className="text-xs text-slate-400 block mb-1">Trigger Event</label>
                <input
                  type="text"
                  value={newHook.trigger}
                  onChange={(e) => setNewHook({ ...newHook, trigger: e.target.value })}
                  placeholder="e.g., file_saved, bash_command"
                  className="w-full px-3 py-2 rounded bg-hive-bg border border-hive-border text-white text-sm focus:border-hive-cyan focus:outline-none"
                />
              </div>

              <div>
                <label className="text-xs text-slate-400 block mb-1">Action</label>
                <input
                  type="text"
                  value={newHook.action}
                  onChange={(e) => setNewHook({ ...newHook, action: e.target.value })}
                  placeholder="e.g., run_linter, log_to_memory"
                  className="w-full px-3 py-2 rounded bg-hive-bg border border-hive-border text-white text-sm focus:border-hive-cyan focus:outline-none"
                />
              </div>

              <div className="pt-2 border-t border-hive-border">
                <p className="text-xs text-slate-500 mb-2">Templates:</p>
                <div className="grid grid-cols-2 gap-2">
                  {Object.entries(HOOK_TEMPLATES).map(([name, template]) => (
                    <button
                      key={name}
                      onClick={() => setNewHook({ ...template, name, enabled: true })}
                      className="text-xs px-2 py-1 rounded bg-hive-border hover:bg-hive-border/80 text-slate-300 transition-colors"
                    >
                      {name}
                    </button>
                  ))}
                </div>
              </div>
            </div>

            <div className="flex gap-2">
              <button
                onClick={() => setShowCreateDialog(false)}
                className="flex-1 px-3 py-2 rounded bg-hive-border text-slate-300 text-sm hover:bg-hive-border/80 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateHook}
                disabled={!newHook.name || !newHook.trigger}
                className="flex-1 px-3 py-2 rounded bg-hive-cyan text-black text-sm font-medium hover:bg-hive-cyan/80 disabled:opacity-50 transition-colors"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Hooks List */}
      <div className="flex-1 overflow-y-auto px-3 py-3">
        {hooks.length > 0 ? (
          <div className="space-y-2">
            {hooks.map((hook) => (
              <div
                key={hook.id}
                draggable
                onDragStart={() => handleDragStart(hook.id)}
                onDragOver={handleDragOver}
                onDrop={() => handleDrop(hook.id)}
                className={`p-3 rounded-lg border transition-all cursor-move ${
                  draggedHookId === hook.id
                    ? "opacity-50 border-hive-cyan"
                    : "border-hive-border/50 hover:border-hive-border"
                } bg-hive-bg`}
              >
                <div className="flex items-start gap-2 mb-1">
                  <label className="flex items-center gap-2 flex-1 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={hook.enabled}
                      onChange={(e) => onToggleHook?.(hook.id, e.target.checked)}
                      className="rounded"
                    />
                    <div>
                      <p className="text-xs font-medium text-slate-200">{hook.name}</p>
                      <p className="text-xs text-slate-500">
                        {hook.type} • {hook.trigger}
                      </p>
                    </div>
                  </label>
                  <button
                    onClick={() => {
                      if (window.confirm(`Delete hook "${hook.name}"?`)) {
                        onDeleteHook?.(hook.id);
                      }
                    }}
                    className="p-1 rounded hover:bg-red-500/20 text-slate-400 hover:text-red-500 transition-colors"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8">
            <p className="text-xs text-slate-500">No hooks configured</p>
          </div>
        )}
      </div>

      {/* Execution Log */}
      {showLogs && (
        <div className="border-t border-hive-border/50 max-h-32 overflow-y-auto bg-hive-bg">
          <div className="px-3 py-2">
            <h4 className="text-xs font-semibold text-slate-300 mb-2">Recent Executions</h4>
            <div className="space-y-1">
              {recentLogs.length > 0 ? (
                recentLogs.map((log, idx) => (
                  <div
                    key={idx}
                    className={`text-xs px-2 py-1 rounded flex items-center gap-2 ${
                      log.success
                        ? "bg-hive-green/10 text-hive-green"
                        : "bg-red-950/50 text-red-400"
                    }`}
                  >
                    <div className={`w-1.5 h-1.5 rounded-full ${log.success ? "bg-hive-green" : "bg-red-500"}`} />
                    <span>{new Date(log.timestamp).toLocaleTimeString()}</span>
                    <span className="opacity-75">{log.message}</span>
                  </div>
                ))
              ) : (
                <p className="text-xs text-slate-500">No executions yet</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
