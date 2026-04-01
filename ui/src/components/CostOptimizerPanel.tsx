import React, { useState, useEffect } from "react";
import { TrendingDown, AlertCircle, CheckCircle } from "lucide-react";

interface CostBreakdown {
  model: string;
  cost: number;
  percentage: number;
  tokenCount: number;
}

interface Recommendation {
  id: string;
  title: string;
  description: string;
  savings: number;
  difficulty: "Easy" | "Medium" | "Advanced";
  action: string;
}

interface DailyCostTrend {
  date: string;
  cost: number;
}

interface CostOptimizerPanelProps {
  sessionCost: number;
  breakdown: CostBreakdown[];
  recommendations: Recommendation[];
  dailyTrend?: DailyCostTrend[];
  compact?: boolean;
}

export const CostOptimizerPanel: React.FC<CostOptimizerPanelProps> = ({
  sessionCost,
  breakdown,
  recommendations,
  dailyTrend = [],
  compact = false,
}) => {
  const [modelRoutingEnabled, setModelRoutingEnabled] = useState(false);
  const [expandedRecs, setExpandedRecs] = useState<Set<string>>(new Set());

  const totalCost = breakdown.reduce((sum, b) => sum + b.cost, 0);
  const potentialSavings = recommendations.reduce((sum, r) => sum + r.savings, 0);

  const toggleRec = (id: string) => {
    const newSet = new Set(expandedRecs);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    setExpandedRecs(newSet);
  };

  if (compact) {
    return (
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-hive-surface border border-hive-border hover:border-hive-yellow transition-colors">
        <TrendingDown className="w-4 h-4 text-hive-yellow" />
        <span className="text-xs font-medium text-slate-300">
          Save ${potentialSavings.toFixed(2)}
        </span>
      </div>
    );
  }

  const maxCost = Math.max(...breakdown.map((b) => b.cost), 0);

  return (
    <div className="rounded-lg border border-hive-border bg-hive-surface p-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-white flex items-center gap-2">
          <TrendingDown className="w-4 h-4 text-hive-yellow" />
          Cost Optimizer
        </h3>
      </div>

      {/* Session Cost Prominent Display */}
      <div className="mb-4 p-3 rounded-lg bg-hive-bg border border-hive-border/50">
        <p className="text-xs text-slate-400 mb-1">Current Session Cost</p>
        <p className="text-2xl font-bold text-hive-cyan">${sessionCost.toFixed(4)}</p>
      </div>

      {/* Potential Savings Callout */}
      {potentialSavings > 0 && (
        <div className="mb-4 p-3 rounded-lg bg-hive-green/10 border border-hive-green/50">
          <p className="text-sm font-semibold text-hive-green mb-1">
            You could save ${potentialSavings.toFixed(2)}
          </p>
          <p className="text-xs text-slate-400">
            {((potentialSavings / totalCost) * 100).toFixed(0)}% cost reduction possible
          </p>
        </div>
      )}

      {/* Cost Breakdown */}
      <div className="mb-4">
        <h4 className="text-xs font-semibold text-slate-300 mb-2">Cost Breakdown</h4>
        <div className="space-y-2">
          {breakdown.map((item) => (
            <div key={item.model}>
              <div className="flex items-center justify-between text-xs mb-1">
                <span className="text-slate-400">{item.model}</span>
                <span className="text-slate-300 font-mono">
                  ${item.cost.toFixed(4)} ({item.percentage.toFixed(0)}%)
                </span>
              </div>
              <div className="w-full h-2 bg-hive-border rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-hive-cyan to-hive-magenta transition-all"
                  style={{
                    width: `${(item.cost / maxCost) * 100}%`,
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Model Routing Toggle */}
      <div className="mb-4 p-3 rounded-lg bg-hive-bg border border-hive-border/50">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={modelRoutingEnabled}
            onChange={(e) => setModelRoutingEnabled(e.target.checked)}
            className="rounded"
          />
          <span className="text-xs font-medium text-slate-300">
            Auto-route to cheapest model
          </span>
        </label>
        <p className="text-xs text-slate-500 mt-1">
          Automatically select the most cost-effective model for your queries
        </p>
      </div>

      {/* Daily Cost Trend */}
      {dailyTrend.length > 0 && (
        <div className="mb-4">
          <h4 className="text-xs font-semibold text-slate-300 mb-2">Daily Cost Trend</h4>
          <div className="flex items-end gap-1 h-16 bg-hive-bg p-2 rounded-lg border border-hive-border/50">
            {dailyTrend.slice(-7).map((item, idx) => (
              <div
                key={idx}
                className="flex-1 bg-hive-cyan rounded-sm transition-all hover:bg-hive-magenta"
                style={{
                  height: `${(item.cost / Math.max(...dailyTrend.map((t) => t.cost), 1)) * 100}%`,
                }}
                title={`${item.date}: $${item.cost.toFixed(2)}`}
              />
            ))}
          </div>
        </div>
      )}

      {/* Recommendations */}
      <div>
        <h4 className="text-xs font-semibold text-slate-300 mb-2">Recommendations</h4>
        <div className="space-y-2">
          {recommendations.length > 0 ? (
            recommendations.map((rec) => (
              <div
                key={rec.id}
                className="p-3 rounded-lg bg-hive-bg border border-hive-border/50 cursor-pointer hover:border-hive-magenta/50 transition-colors"
                onClick={() => toggleRec(rec.id)}
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex-1">
                    <p className="text-xs font-medium text-slate-200">{rec.title}</p>
                    <p className="text-xs text-slate-500">
                      Save ${rec.savings.toFixed(2)}
                    </p>
                  </div>
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      rec.difficulty === "Easy"
                        ? "bg-hive-green/20 text-hive-green"
                        : rec.difficulty === "Medium"
                          ? "bg-hive-yellow/20 text-hive-yellow"
                          : "bg-hive-magenta/20 text-hive-magenta"
                    }`}
                  >
                    {rec.difficulty}
                  </span>
                </div>
                {expandedRecs.has(rec.id) && (
                  <div className="pt-2 border-t border-hive-border/30">
                    <p className="text-xs text-slate-400 mb-2">{rec.description}</p>
                    <button className="text-xs px-2 py-1 rounded bg-hive-cyan text-black font-medium hover:bg-hive-cyan/80 transition-colors">
                      {rec.action}
                    </button>
                  </div>
                )}
              </div>
            ))
          ) : (
            <div className="p-3 rounded-lg bg-hive-bg border border-hive-border/50">
              <p className="text-xs text-slate-500">
                No optimization opportunities at this time
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
