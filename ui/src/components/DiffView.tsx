import React, { useState, useEffect } from "react";
import { ChevronDown, ChevronUp, Copy, Check } from "lucide-react";

interface FileDiff {
  filename: string;
  additions: number;
  deletions: number;
  hunks: Hunk[];
  isStreaming?: boolean;
}

interface Hunk {
  id: string;
  oldStart: number;
  newStart: number;
  oldLines: number;
  newLines: number;
  lines: DiffLine[];
}

interface DiffLine {
  type: "add" | "remove" | "context";
  content: string;
  lineNumber?: number;
}

interface DiffViewProps {
  files: FileDiff[];
  compact?: boolean;
  onViewChange?: (view: "split" | "unified") => void;
}

export const DiffView: React.FC<DiffViewProps> = ({
  files,
  compact = false,
  onViewChange,
}) => {
  const [viewMode, setViewMode] = useState<"split" | "unified">("unified");
  const [expandedHunks, setExpandedHunks] = useState<Set<string>>(new Set());
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const toggleHunk = (hunkId: string) => {
    const newExpanded = new Set(expandedHunks);
    if (newExpanded.has(hunkId)) {
      newExpanded.delete(hunkId);
    } else {
      newExpanded.add(hunkId);
    }
    setExpandedHunks(newExpanded);
  };

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const totalAdditions = files.reduce((sum, f) => sum + f.additions, 0);
  const totalDeletions = files.reduce((sum, f) => sum + f.deletions, 0);

  if (compact) {
    return (
      <div className="px-3 py-2 rounded-lg bg-hive-surface border border-hive-border text-xs">
        <div className="flex items-center gap-3">
          <span className="text-slate-400">Changes:</span>
          <span className="text-hive-green">+{totalAdditions}</span>
          <span className="text-red-500">-{totalDeletions}</span>
          <span className="text-slate-500">in {files.length} files</span>
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-hive-border bg-hive-surface overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b border-hive-border/50 bg-hive-bg">
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-4">
            <h3 className="text-sm font-semibold text-white">File Changes</h3>
            <div className="flex items-center gap-3 text-xs text-slate-400">
              <span className="text-hive-green">+{totalAdditions}</span>
              <span className="text-red-500">-{totalDeletions}</span>
              <span>in {files.length} files</span>
            </div>
          </div>

          <div className="flex gap-2">
            <button
              onClick={() => {
                setViewMode("unified");
                onViewChange?.("unified");
              }}
              className={`px-3 py-1 rounded text-xs transition-colors ${
                viewMode === "unified"
                  ? "bg-hive-cyan text-black"
                  : "bg-hive-border text-slate-300 hover:bg-hive-border/80"
              }`}
            >
              Unified
            </button>
            <button
              onClick={() => {
                setViewMode("split");
                onViewChange?.("split");
              }}
              className={`px-3 py-1 rounded text-xs transition-colors ${
                viewMode === "split"
                  ? "bg-hive-cyan text-black"
                  : "bg-hive-border text-slate-300 hover:bg-hive-border/80"
              }`}
            >
              Split
            </button>
          </div>
        </div>
      </div>

      {/* Files */}
      <div className="divide-y divide-hive-border/50">
        {files.map((file) => (
          <div key={file.filename} className="bg-hive-surface">
            {/* File Tab */}
            <div className="px-4 py-3 bg-hive-bg border-b border-hive-border/50">
              <div className="flex items-center justify-between">
                <span className="text-sm font-mono text-hive-cyan">
                  {file.filename}
                </span>
                <div className="flex items-center gap-2 text-xs text-slate-400">
                  <span className="text-hive-green">+{file.additions}</span>
                  <span className="text-red-500">-{file.deletions}</span>
                  {file.isStreaming && (
                    <span className="text-hive-magenta animate-pulse">
                      Streaming...
                    </span>
                  )}
                </div>
              </div>
            </div>

            {/* Hunks */}
            <div className="divide-y divide-hive-border/30">
              {file.hunks.map((hunk) => {
                const isExpanded = expandedHunks.has(hunk.id);
                return (
                  <div key={hunk.id}>
                    {/* Hunk Header */}
                    <button
                      onClick={() => toggleHunk(hunk.id)}
                      className="w-full px-4 py-2 flex items-center gap-2 bg-hive-surface hover:bg-hive-border/30 transition-colors text-xs text-slate-500 font-mono"
                    >
                      {isExpanded ? (
                        <ChevronUp className="w-3 h-3" />
                      ) : (
                        <ChevronDown className="w-3 h-3" />
                      )}
                      <span>
                        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},
                        {hunk.newLines} @@
                      </span>
                    </button>

                    {/* Hunk Content */}
                    {isExpanded && (
                      <div className="bg-hive-bg font-mono text-xs overflow-x-auto">
                        {hunk.lines.map((line, idx) => {
                          const lineId = `${hunk.id}-${idx}`;
                          return (
                            <div
                              key={idx}
                              className={`flex ${
                                line.type === "add"
                                  ? "bg-green-950/40"
                                  : line.type === "remove"
                                    ? "bg-red-950/40"
                                    : ""
                              }`}
                            >
                              <div className="w-10 flex-shrink-0 text-slate-600 text-right px-2 py-1 border-r border-hive-border/30">
                                {line.lineNumber}
                              </div>
                              <div
                                className={`flex-1 px-3 py-1 ${
                                  line.type === "add"
                                    ? "text-hive-green"
                                    : line.type === "remove"
                                      ? "text-red-500"
                                      : "text-slate-400"
                                }`}
                              >
                                <span className="select-none">
                                  {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
                                </span>
                                <span className="select-text">
                                  {line.content}
                                </span>
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
