import React, { useState } from "react";
import { ChevronDown, CheckCircle, AlertCircle, Clock } from "lucide-react";

interface ToolPanelProps {
  toolName: string;
  status: "running" | "success" | "error";
  output: string;
  error?: string;
}

export const ToolPanel: React.FC<ToolPanelProps> = ({
  toolName,
  status,
  output,
  error,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);

  const statusIcon =
    status === "running" ? (
      <Clock className="w-4 h-4 text-hive-amber animate-spin" />
    ) : status === "success" ? (
      <CheckCircle className="w-4 h-4 text-green-500" />
    ) : (
      <AlertCircle className="w-4 h-4 text-red-500" />
    );

  const statusText =
    status === "running"
      ? "Running"
      : status === "success"
        ? "Success"
        : "Error";

  return (
    <div className={`tool-card ${status} animate-slide-in`}>
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between"
      >
        <div className="flex items-center gap-2">
          {statusIcon}
          <span className="font-semibold text-sm">{toolName}</span>
          <span className="text-xs text-slate-600 dark:text-slate-400">
            {statusText}
          </span>
        </div>
        <ChevronDown
          className={`w-4 h-4 transition-transform ${
            isExpanded ? "rotate-180" : ""
          }`}
        />
      </button>

      {isExpanded && (
        <div className="mt-3 text-xs">
          {error && (
            <div className="mb-3 p-2 rounded bg-red-100 dark:bg-red-950 text-red-800 dark:text-red-200">
              {error}
            </div>
          )}
          <div className="bg-hive-bg-light dark:bg-slate-900 p-3 rounded overflow-x-auto max-h-48 overflow-y-auto">
            <pre className="font-mono text-slate-900 dark:text-slate-100 whitespace-pre-wrap break-words">
              {output}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
};
