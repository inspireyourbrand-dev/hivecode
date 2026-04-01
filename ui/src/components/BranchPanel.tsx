import React, { useState } from "react";
import { GitBranch, Plus, Trash2, Eye } from "lucide-react";

interface Branch {
  id: string;
  name: string;
  parentId?: string;
  messageCount: number;
  cost: number;
  model: string;
  createdAt: string;
  isCurrent?: boolean;
}

interface BranchPanelProps {
  branches: Branch[];
  currentBranchId: string;
  onSwitchBranch?: (branchId: string) => void;
  onForkBranch?: (fromId: string) => void;
  onDeleteBranch?: (branchId: string) => void;
  onCompareBranches?: (branchId1: string, branchId2: string) => void;
}

export const BranchPanel: React.FC<BranchPanelProps> = ({
  branches,
  currentBranchId,
  onSwitchBranch,
  onForkBranch,
  onDeleteBranch,
  onCompareBranches,
}) => {
  const [selectedForComparison, setSelectedForComparison] = useState<string | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newBranchName, setNewBranchName] = useState("");

  const buildTree = (parentId?: string): Branch[] => {
    return branches.filter((b) => b.parentId === parentId);
  };

  const renderBranch = (branch: Branch, depth: number = 0) => {
    const children = buildTree(branch.id);
    const isComparing =
      selectedForComparison === branch.id ||
      (selectedForComparison && selectedForComparison === currentBranchId && branch.id === currentBranchId);

    return (
      <div key={branch.id}>
        {/* Branch Item */}
        <div
          className={`px-3 py-2 rounded-lg mb-2 transition-colors ${
            branch.isCurrent
              ? "bg-hive-cyan/20 border border-hive-cyan"
              : "hover:bg-hive-border/50"
          }`}
          style={{ marginLeft: `${depth * 12}px` }}
        >
          <div className="flex items-center gap-2 mb-1">
            <GitBranch
              className={`w-4 h-4 flex-shrink-0 ${
                branch.isCurrent ? "text-hive-cyan" : "text-hive-magenta"
              }`}
            />
            <button
              onClick={() => {
                onSwitchBranch?.(branch.id);
              }}
              className="flex-1 text-left text-sm font-medium text-slate-200 hover:text-hive-cyan transition-colors"
            >
              {branch.name}
            </button>
            <div className="flex items-center gap-1 flex-shrink-0">
              {selectedForComparison === branch.id && (
                <Eye className="w-3 h-3 text-hive-yellow" />
              )}
              {branch.isCurrent && (
                <span className="text-xs bg-hive-cyan text-black px-2 py-0.5 rounded">
                  current
                </span>
              )}
            </div>
          </div>

          {/* Metadata */}
          <div className="flex items-center gap-3 text-xs text-slate-400 ml-6">
            <span>{branch.messageCount} messages</span>
            <span>${branch.cost.toFixed(4)}</span>
            <span className="text-hive-green text-xs">{branch.model}</span>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2 mt-2 ml-6">
            <button
              onClick={() => {
                if (selectedForComparison && selectedForComparison !== branch.id) {
                  onCompareBranches?.(selectedForComparison, branch.id);
                } else {
                  setSelectedForComparison(branch.id);
                }
              }}
              className="px-2 py-1 text-xs rounded bg-hive-border hover:bg-hive-cyan/20 text-slate-300 hover:text-hive-cyan transition-colors"
            >
              {selectedForComparison === branch.id ? "Compare" : "Select"}
            </button>
            {!branch.isCurrent && (
              <>
                <button
                  onClick={() => onForkBranch?.(branch.id)}
                  className="px-2 py-1 text-xs rounded bg-hive-border hover:bg-hive-magenta/20 text-slate-300 hover:text-hive-magenta transition-colors"
                >
                  <Plus className="w-3 h-3" />
                </button>
                <button
                  onClick={() => {
                    if (window.confirm(`Delete branch "${branch.name}"?`)) {
                      onDeleteBranch?.(branch.id);
                    }
                  }}
                  className="px-2 py-1 text-xs rounded bg-hive-border hover:bg-red-500/20 text-slate-300 hover:text-red-500 transition-colors"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </>
            )}
          </div>
        </div>

        {/* Children */}
        {children.map((child) => renderBranch(child, depth + 1))}
      </div>
    );
  };

  const rootBranches = buildTree(undefined);

  return (
    <div className="h-full flex flex-col rounded-lg border border-hive-border bg-hive-surface">
      {/* Header */}
      <div className="px-4 py-3 border-b border-hive-border/50 flex items-center justify-between flex-shrink-0">
        <h3 className="text-sm font-semibold text-white flex items-center gap-2">
          <GitBranch className="w-4 h-4 text-hive-magenta" />
          Conversation Branches
        </h3>
        <button
          onClick={() => setShowCreateDialog(true)}
          className="p-1.5 rounded hover:bg-hive-border transition-colors text-hive-cyan"
          title="New branch"
        >
          <Plus className="w-4 h-4" />
        </button>
      </div>

      {/* Create Branch Dialog */}
      {showCreateDialog && (
        <div className="absolute inset-0 bg-black/50 flex items-center justify-center z-50 rounded-lg">
          <div className="bg-hive-surface border border-hive-cyan p-4 rounded-lg max-w-sm w-full mx-4">
            <h4 className="text-sm font-semibold text-white mb-3">Create New Branch</h4>
            <input
              type="text"
              value={newBranchName}
              onChange={(e) => setNewBranchName(e.target.value)}
              placeholder="Branch name..."
              className="w-full px-3 py-2 rounded bg-hive-bg border border-hive-border text-white text-sm mb-3 focus:border-hive-cyan focus:outline-none"
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  setShowCreateDialog(false);
                  setNewBranchName("");
                }
              }}
              autoFocus
            />
            <div className="flex gap-2">
              <button
                onClick={() => setShowCreateDialog(false)}
                className="flex-1 px-3 py-2 rounded bg-hive-border text-slate-300 text-sm hover:bg-hive-border/80 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  if (newBranchName.trim()) {
                    onForkBranch?.(currentBranchId);
                    setNewBranchName("");
                  }
                  setShowCreateDialog(false);
                }}
                className="flex-1 px-3 py-2 rounded bg-hive-cyan text-black text-sm font-medium hover:bg-hive-cyan/80 transition-colors"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Branch Tree */}
      <div className="flex-1 overflow-y-auto px-3 py-3">
        {rootBranches.length > 0 ? (
          rootBranches.map((branch) => renderBranch(branch))
        ) : (
          <div className="text-center py-8">
            <p className="text-xs text-slate-500">No branches yet</p>
          </div>
        )}
      </div>

      {/* Stats Footer */}
      <div className="px-3 py-3 border-t border-hive-border/50 text-xs text-slate-400 flex-shrink-0">
        <p>{branches.length} total branches</p>
      </div>
    </div>
  );
};
