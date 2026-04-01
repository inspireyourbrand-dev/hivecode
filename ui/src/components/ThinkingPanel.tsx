import React, { useState, useEffect } from "react";
import { ChevronDown, Zap, Brain, Lightbulb } from "lucide-react";

interface ThinkingPanelProps {
  thinking: string;
  isStreaming?: boolean;
  thinkingType?: "reasoning" | "planning" | "analysis";
  tokenCount?: number;
  timeMs?: number;
}

export const ThinkingPanel: React.FC<ThinkingPanelProps> = ({
  thinking,
  isStreaming = false,
  thinkingType = "reasoning",
  tokenCount = 0,
  timeMs = 0,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [displayedText, setDisplayedText] = useState("");

  useEffect(() => {
    if (isStreaming && thinking.length > displayedText.length) {
      // Simulate streaming by gradually showing text
      const timeout = setTimeout(() => {
        setDisplayedText(thinking.slice(0, displayedText.length + 10));
      }, 20);
      return () => clearTimeout(timeout);
    }
  }, [thinking, displayedText, isStreaming]);

  const getIcon = () => {
    switch (thinkingType) {
      case "planning":
        return <Zap className="w-4 h-4 text-hive-yellow" />;
      case "analysis":
        return <Lightbulb className="w-4 h-4 text-hive-cyan" />;
      default:
        return <Brain className="w-4 h-4 text-hive-magenta" />;
    }
  };

  const summaryText =
    displayedText.length > 100 ? displayedText.slice(0, 100) + "..." : displayedText;

  return (
    <div
      className={`mb-3 rounded-lg border transition-all ${
        isExpanded
          ? "border-hive-magenta bg-hive-bg"
          : "border-hive-border bg-hive-surface"
      } ${isStreaming ? "shadow-lg shadow-hive-magenta/20" : ""}`}
    >
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 flex items-center gap-3 hover:bg-hive-border/50 transition-colors"
      >
        <ChevronDown
          className={`w-4 h-4 text-hive-magenta transition-transform ${
            isExpanded ? "rotate-180" : ""
          }`}
        />

        <div className="flex items-center gap-2 flex-1 min-w-0">
          {getIcon()}
          <span className="text-sm font-medium text-slate-300">
            {isStreaming ? (
              <>
                <span className="inline-block animate-pulse">Thinking...</span>
              </>
            ) : (
              <span>Thinking</span>
            )}
          </span>
        </div>

        <div className="flex items-center gap-2 text-xs text-slate-500 flex-shrink-0">
          {tokenCount > 0 && <span>{tokenCount} tokens</span>}
          {timeMs > 0 && <span>{timeMs}ms</span>}
        </div>
      </button>

      {!isExpanded && summaryText && (
        <div className="px-4 pb-3">
          <p className="text-xs text-slate-400 line-clamp-2">{summaryText}</p>
        </div>
      )}

      {isExpanded && (
        <div className="px-4 pb-4 border-t border-hive-border/50">
          <div className="max-h-96 overflow-y-auto">
            <p className="text-sm text-slate-300 leading-relaxed whitespace-pre-wrap">
              {isStreaming && displayedText.length < thinking.length
                ? displayedText + "▌"
                : displayedText || thinking}
            </p>
          </div>
          {isStreaming && (
            <div className="mt-2 text-xs text-hive-magenta animate-pulse">
              Still thinking...
            </div>
          )}
        </div>
      )}
    </div>
  );
};
