import React, { useState } from "react";
import { useChatStore } from "@/stores/chatStore";
import { useNotification } from "./NotificationToast";
import { Zap, Loader } from "lucide-react";

interface CompactState {
  isCompacting: boolean;
  tokensCompressed?: number;
  messagesCompressed?: number;
}

export const CompactButton: React.FC = () => {
  const [state, setState] = useState<CompactState>({
    isCompacting: false,
  });
  const [showConfirm, setShowConfirm] = useState(false);
  const messages = useChatStore((state) => state.messages);
  const notification = useNotification();

  const handleCompact = async () => {
    setState({ isCompacting: true });
    setShowConfirm(false);

    try {
      // Simulate compaction delay
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Mock results
      const compressed = Math.floor(messages.length * 0.3);
      const tokens = Math.floor(Math.random() * 5000) + 2000;

      setState({
        isCompacting: false,
        tokensCompressed: tokens,
        messagesCompressed: compressed,
      });

      notification.success(
        "Conversation Compacted",
        `Compressed ${compressed} messages, saved ~${tokens} tokens`
      );

      // Clear result after 5 seconds
      setTimeout(() => {
        setState({ isCompacting: false });
      }, 5000);
    } catch (error) {
      notification.error(
        "Compaction Failed",
        error instanceof Error ? error.message : "Unknown error"
      );
      setState({ isCompacting: false });
    }
  };

  const tokenCount = messages.reduce(
    (acc, msg) => acc + JSON.stringify(msg).length / 4,
    0
  );

  return (
    <>
      {/* Compact Button */}
      <div className="relative group">
        <button
          onClick={() => setShowConfirm(true)}
          disabled={state.isCompacting || messages.length === 0}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-hive-surface border border-hive-border hover:border-hive-cyan hover:bg-hive-cyan/5 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
          title="Compact conversation"
        >
          {state.isCompacting ? (
            <>
              <Loader className="w-4 h-4 text-hive-cyan animate-spin" />
              <span className="text-xs font-medium">Compacting...</span>
            </>
          ) : (
            <>
              <Zap className="w-4 h-4 text-hive-yellow" />
              <span className="text-xs font-medium">
                {Math.round(tokenCount)} tokens
              </span>
            </>
          )}
        </button>

        {/* Tooltip */}
        {!state.isCompacting && (
          <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
            <div className="px-2 py-1 bg-hive-bg rounded text-xs text-slate-300 border border-hive-border whitespace-nowrap">
              Compress conversation history
            </div>
          </div>
        )}

        {/* Result Message */}
        {state.tokensCompressed && !state.isCompacting && (
          <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-2 bg-hive-green/10 border border-hive-green rounded text-xs text-hive-green whitespace-nowrap animate-slide-in">
            Saved {state.tokensCompressed} tokens
          </div>
        )}
      </div>

      {/* Confirmation Dialog */}
      {showConfirm && (
        <div
          className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center"
          onClick={() => setShowConfirm(false)}
        >
          <div
            className="bg-hive-surface border border-hive-cyan rounded-lg p-6 max-w-sm mx-4"
            onClick={(e) => e.stopPropagation()}
          >
            <h2 className="text-lg font-semibold text-white mb-2">
              Compact Conversation?
            </h2>
            <p className="text-sm text-slate-400 mb-6">
              This will summarize earlier messages to reduce token usage. You'll
              still have access to the full conversation.
            </p>

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowConfirm(false)}
                className="px-4 py-2 rounded-lg border border-hive-border hover:bg-white/5 transition-colors text-sm font-medium"
              >
                Cancel
              </button>
              <button
                onClick={handleCompact}
                className="px-4 py-2 rounded-lg bg-gradient-to-r from-hive-cyan to-hive-magenta text-white font-medium text-sm hover:shadow-lg hover:shadow-hive-cyan/50 transition-all"
              >
                Compact
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
