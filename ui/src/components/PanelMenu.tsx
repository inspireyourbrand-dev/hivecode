import React, { useState, useRef, useEffect } from "react";
import { LayoutGrid, X } from "lucide-react";
import { BranchPanel } from "./BranchPanel";
import { CostOptimizerPanel } from "./CostOptimizerPanel";
import { HooksPanel } from "./HooksPanel";
import { SessionReplayPlayer } from "./SessionReplayPlayer";
import { ProjectInstructions } from "./ProjectInstructions";

interface PanelMenuProps {
  // Branch Panel props
  branches?: any[];
  currentBranchId?: string;
  onSwitchBranch?: (branchId: string) => void;
  onForkBranch?: (fromId: string) => void;
  onDeleteBranch?: (branchId: string) => void;
  onCompareBranches?: (branchId1: string, branchId2: string) => void;

  // Cost Optimizer props
  sessionCost?: number;
  breakdown?: any[];
  recommendations?: any[];
  dailyTrend?: any[];

  // Hooks Panel props
  hooks?: any[];
  hookLogs?: any[];
  onToggleHook?: (hookId: string, enabled: boolean) => void;
  onCreateHook?: (hook: any) => void;
  onDeleteHook?: (hookId: string) => void;
  onReorderHooks?: (hookIds: string[]) => void;

  // Session Replay props
  events?: any[];
  onExportSession?: (format: "markdown" | "json") => void;

  // Project Instructions props
  projectInstructionsContent?: string;
  onSaveProjectInstructions?: (content: string) => Promise<void>;
  onLoadProjectTemplate?: () => Promise<string>;
}

export const PanelMenu: React.FC<PanelMenuProps> = ({
  branches = [],
  currentBranchId = "",
  onSwitchBranch,
  onForkBranch,
  onDeleteBranch,
  onCompareBranches,
  sessionCost = 0,
  breakdown = [],
  recommendations = [],
  dailyTrend = [],
  hooks = [],
  hookLogs = [],
  onToggleHook,
  onCreateHook,
  onDeleteHook,
  onReorderHooks,
  events = [],
  onExportSession,
  projectInstructionsContent = "",
  onSaveProjectInstructions,
  onLoadProjectTemplate,
}) => {
  const [menuOpen, setMenuOpen] = useState(false);
  const [openPanel, setOpenPanel] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    };

    if (menuOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [menuOpen]);

  const menuItems = [
    { id: "branches", label: "Branches", icon: "🌳" },
    { id: "costs", label: "Cost Optimizer", icon: "💰" },
    { id: "hooks", label: "Hooks", icon: "⚡" },
    { id: "replay", label: "Session Replay", icon: "▶️" },
    { id: "instructions", label: "Project Config", icon: "📋" },
  ];

  return (
    <>
      {/* Menu Button */}
      <div className="relative" ref={menuRef}>
        <button
          onClick={() => setMenuOpen(!menuOpen)}
          className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded-lg transition-colors relative group"
          title="Advanced panels"
        >
          <LayoutGrid className="w-5 h-5 text-slate-700 dark:text-slate-300" />
        </button>

        {/* Dropdown Menu */}
        {menuOpen && (
          <div className="absolute right-0 top-full mt-2 w-48 bg-white dark:bg-hive-surface rounded-lg shadow-lg border border-hive-border-light dark:border-hive-border z-50">
            <div className="py-2">
              {menuItems.map((item) => (
                <button
                  key={item.id}
                  onClick={() => {
                    setOpenPanel(item.id);
                    setMenuOpen(false);
                  }}
                  className="w-full text-left px-4 py-2 text-sm text-slate-900 dark:text-white hover:bg-hive-bg-light dark:hover:bg-hive-border transition-colors flex items-center gap-2"
                >
                  <span>{item.icon}</span>
                  <span>{item.label}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Branches Panel */}
      {openPanel === "branches" && (
        <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setOpenPanel(null)}>
          <div
            className="absolute right-0 top-0 bottom-0 w-full max-w-md bg-hive-surface border-l border-hive-border flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-hive-border flex-shrink-0">
              <h2 className="text-lg font-semibold text-white">Conversation Branches</h2>
              <button
                onClick={() => setOpenPanel(null)}
                className="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            {/* Content */}
            <div className="flex-1 overflow-hidden">
              <BranchPanel
                branches={branches}
                currentBranchId={currentBranchId}
                onSwitchBranch={onSwitchBranch}
                onForkBranch={onForkBranch}
                onDeleteBranch={onDeleteBranch}
                onCompareBranches={onCompareBranches}
              />
            </div>
          </div>
        </div>
      )}

      {/* Cost Optimizer Panel */}
      {openPanel === "costs" && (
        <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setOpenPanel(null)}>
          <div
            className="absolute right-0 top-0 bottom-0 w-full max-w-md bg-hive-surface border-l border-hive-border flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-hive-border flex-shrink-0">
              <h2 className="text-lg font-semibold text-white">Cost Optimizer</h2>
              <button
                onClick={() => setOpenPanel(null)}
                className="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            {/* Content */}
            <div className="flex-1 overflow-y-auto p-4">
              <CostOptimizerPanel
                sessionCost={sessionCost}
                breakdown={breakdown}
                recommendations={recommendations}
                dailyTrend={dailyTrend}
              />
            </div>
          </div>
        </div>
      )}

      {/* Hooks Panel */}
      {openPanel === "hooks" && (
        <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setOpenPanel(null)}>
          <div
            className="absolute right-0 top-0 bottom-0 w-full max-w-md bg-hive-surface border-l border-hive-border flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-hive-border flex-shrink-0">
              <h2 className="text-lg font-semibold text-white">Hooks</h2>
              <button
                onClick={() => setOpenPanel(null)}
                className="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            {/* Content */}
            <div className="flex-1 overflow-hidden">
              <HooksPanel
                hooks={hooks}
                logs={hookLogs}
                onToggleHook={onToggleHook}
                onCreateHook={onCreateHook}
                onDeleteHook={onDeleteHook}
                onReorderHooks={onReorderHooks}
              />
            </div>
          </div>
        </div>
      )}

      {/* Session Replay Panel */}
      {openPanel === "replay" && (
        <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setOpenPanel(null)}>
          <div
            className="absolute right-0 top-0 bottom-0 w-full max-w-2xl bg-hive-surface border-l border-hive-border flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-hive-border flex-shrink-0">
              <h2 className="text-lg font-semibold text-white">Session Replay</h2>
              <button
                onClick={() => setOpenPanel(null)}
                className="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            {/* Content */}
            <div className="flex-1 overflow-hidden">
              <SessionReplayPlayer events={events} onExport={onExportSession} />
            </div>
          </div>
        </div>
      )}

      {/* Project Instructions Panel */}
      {openPanel === "instructions" && (
        <div className="fixed inset-0 z-40 bg-black/50" onClick={() => setOpenPanel(null)}>
          <div
            className="absolute right-0 top-0 bottom-0 w-full max-w-2xl bg-hive-surface border-l border-hive-border flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-hive-border flex-shrink-0">
              <h2 className="text-lg font-semibold text-white">Project Configuration</h2>
              <button
                onClick={() => setOpenPanel(null)}
                className="p-1 hover:bg-white/10 rounded transition-colors"
              >
                <X className="w-5 h-5 text-slate-400" />
              </button>
            </div>
            {/* Content */}
            <div className="flex-1 overflow-hidden">
              <ProjectInstructions
                content={projectInstructionsContent}
                onSave={onSaveProjectInstructions}
                onLoadTemplate={onLoadProjectTemplate}
              />
            </div>
          </div>
        </div>
      )}
    </>
  );
};
