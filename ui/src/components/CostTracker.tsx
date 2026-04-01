import React, { useEffect, useState } from "react";
import { TrendingUp, AlertTriangle } from "lucide-react";

interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  session_cost: number;
  total_cost: number;
}

interface CostTrackerProps {
  compact?: boolean;
}

const calculateCostPercentage = (
  current: number,
  limit: number
): number => {
  if (limit === 0) return 0;
  return Math.min((current / limit) * 100, 100);
};

const getWarningLevel = (percentage: number): "none" | "warning" | "critical" => {
  if (percentage >= 100) return "critical";
  if (percentage >= 80) return "warning";
  return "none";
};

export const CostTracker: React.FC<CostTrackerProps> = ({ compact = true }) => {
  const [usage, setUsage] = useState<TokenUsage>({
    input_tokens: 0,
    output_tokens: 0,
    session_cost: 0,
    total_cost: 0,
  });
  const [spendingLimit] = useState<number>(10); // Default $10 limit
  const costPercentage = calculateCostPercentage(usage.total_cost, spendingLimit);
  const warningLevel = getWarningLevel(costPercentage);

  // In a real app, this would fetch from Tauri
  useEffect(() => {
    // Mock data for demo
    const timer = setInterval(() => {
      setUsage((prev) => ({
        ...prev,
        input_tokens: Math.floor(Math.random() * 5000),
        output_tokens: Math.floor(Math.random() * 3000),
        session_cost: parseFloat((Math.random() * 2).toFixed(4)),
        total_cost: parseFloat((Math.random() * 8).toFixed(4)),
      }));
    }, 5000);

    return () => clearInterval(timer);
  }, []);

  if (compact) {
    // Compact header version
    return (
      <div className="flex items-center gap-3 px-3 py-1.5 rounded-lg bg-hive-surface border border-hive-border hover:border-hive-cyan transition-colors">
        <div className="flex items-center gap-2">
          <TrendingUp className="w-4 h-4 text-hive-cyan" />
          <span className="text-xs font-mono text-slate-400">
            ${usage.total_cost.toFixed(4)}
          </span>
        </div>

        {warningLevel !== "none" && (
          <div
            className={`flex items-center gap-1 ${
              warningLevel === "critical"
                ? "text-red-500"
                : "text-hive-yellow"
            }`}
          >
            <AlertTriangle className="w-3 h-3" />
          </div>
        )}

        <div className="h-1 w-16 bg-hive-border rounded-full overflow-hidden">
          <div
            className={`h-full transition-all ${
              warningLevel === "critical"
                ? "bg-red-500"
                : warningLevel === "warning"
                  ? "bg-hive-yellow"
                  : "bg-hive-cyan"
            }`}
            style={{ width: `${Math.min(costPercentage, 100)}%` }}
          />
        </div>
      </div>
    );
  }

  // Expanded version
  return (
    <div className="p-4 rounded-lg border border-hive-border bg-hive-surface">
      <div className="mb-4">
        <h3 className="text-sm font-semibold text-white mb-1">
          Cost Tracker
        </h3>
        <p className="text-xs text-slate-400">
          Session: ${usage.session_cost.toFixed(4)} | Total: ${usage.total_cost.toFixed(4)}
        </p>
      </div>

      {/* Progress Bar */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs text-slate-400">
            Spending Limit: ${spendingLimit.toFixed(2)}
          </span>
          <span className={`text-xs font-semibold ${
            warningLevel === "critical"
              ? "text-red-500"
              : warningLevel === "warning"
                ? "text-hive-yellow"
                : "text-hive-cyan"
          }`}>
            {costPercentage.toFixed(1)}%
          </span>
        </div>

        <div className="h-2 bg-hive-border rounded-full overflow-hidden">
          <div
            className={`h-full transition-all ${
              warningLevel === "critical"
                ? "bg-red-500"
                : warningLevel === "warning"
                  ? "bg-hive-yellow"
                  : "bg-hive-cyan"
            }`}
            style={{ width: `${Math.min(costPercentage, 100)}%` }}
          />
        </div>
      </div>

      {/* Token Counts */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <div className="p-3 rounded-lg bg-hive-bg border border-hive-border">
          <p className="text-xs text-slate-400 mb-1">Input Tokens</p>
          <p className="text-sm font-mono text-hive-cyan">
            {usage.input_tokens.toLocaleString()}
          </p>
        </div>
        <div className="p-3 rounded-lg bg-hive-bg border border-hive-border">
          <p className="text-xs text-slate-400 mb-1">Output Tokens</p>
          <p className="text-sm font-mono text-hive-green">
            {usage.output_tokens.toLocaleString()}
          </p>
        </div>
      </div>

      {/* Warning Messages */}
      {warningLevel === "critical" && (
        <div className="p-3 rounded-lg bg-red-950 border border-red-500">
          <p className="text-xs text-red-200">
            You have reached your spending limit. Consider adjusting your limit or reviewing your usage.
          </p>
        </div>
      )}

      {warningLevel === "warning" && (
        <div className="p-3 rounded-lg bg-yellow-950 border border-hive-yellow">
          <p className="text-xs text-yellow-200">
            You are approaching your spending limit (${spendingLimit.toFixed(2)}).
          </p>
        </div>
      )}
    </div>
  );
};
