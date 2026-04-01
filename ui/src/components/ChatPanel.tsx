import React, { useEffect, useRef } from "react";
import { useChatStore } from "@/stores/chatStore";
import { useAppStore } from "@/stores/appStore";
import { MessageBubble } from "./MessageBubble";
import { InputArea } from "./InputArea";
import { useAutoScroll } from "@/hooks/useAutoScroll";
import { Message } from "@/lib/types";

export const ChatPanel: React.FC = () => {
  const messages = useChatStore((state) => state.messages);
  const isStreaming = useChatStore((state) => state.isStreaming);
  const currentStreamContent = useChatStore(
    (state) => state.currentStreamContent
  );
  const scrollRef = useAutoScroll(messages.length);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [messages, currentStreamContent]);

  const streamingMessage: Message | null = isStreaming
    ? {
        id: "streaming",
        role: "assistant",
        content: currentStreamContent
          ? [{ type: "text", text: currentStreamContent }]
          : [],
        timestamp: new Date().toISOString(),
      }
    : null;

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Messages Container */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto px-4 py-6 space-y-2"
      >
        {messages.length === 0 && !isStreaming ? (
          <div className="h-full flex flex-col items-center justify-center text-center">
            <div className="mb-6">
              <div className="text-6xl mb-4">🐝</div>
              <h2 className="text-2xl font-bold text-slate-900 dark:text-white mb-2">
                Welcome to HiveCode
              </h2>
              <p className="text-slate-600 dark:text-slate-400 max-w-md">
                Your AI-powered coding assistant. Start by asking a question or
                describing what you'd like to build.
              </p>
            </div>

            <div className="grid grid-cols-1 gap-3 mt-8 max-w-2xl w-full">
              <div className="p-4 rounded-lg bg-hive-bg-light dark:bg-hive-surface border border-hive-border-light dark:border-hive-border hover:border-hive-cyan cursor-pointer transition-colors">
                <p className="font-medium text-sm">How can I help?</p>
                <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                  Ask me to write code, debug, refactor, or explain
                </p>
              </div>
              <div className="p-4 rounded-lg bg-hive-bg-light dark:bg-hive-surface border border-hive-border-light dark:border-hive-border hover:border-hive-cyan cursor-pointer transition-colors">
                <p className="font-medium text-sm">Project explorer</p>
                <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                  Open a project from the sidebar to analyze your codebase
                </p>
              </div>
              <div className="p-4 rounded-lg bg-hive-bg-light dark:bg-hive-surface border border-hive-border-light dark:border-hive-border hover:border-hive-cyan cursor-pointer transition-colors">
                <p className="font-medium text-sm">Multi-model support</p>
                <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                  Switch between different AI models in the sidebar
                </p>
              </div>
            </div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <MessageBubble key={message.id} message={message} />
            ))}
            {streamingMessage && (
              <MessageBubble message={streamingMessage} />
            )}
            {isStreaming && !currentStreamContent && (
              <div className="flex gap-1 mt-4">
                <div className="w-2 h-2 bg-hive-cyan rounded-full animate-pulse-dot" />
                <div
                  className="w-2 h-2 bg-hive-cyan rounded-full animate-pulse-dot"
                  style={{ animationDelay: "0.2s" }}
                />
                <div
                  className="w-2 h-2 bg-hive-cyan rounded-full animate-pulse-dot"
                  style={{ animationDelay: "0.4s" }}
                />
              </div>
            )}
            <div ref={scrollRef} />
          </>
        )}
      </div>

      {/* Input Area */}
      <div className="border-t border-hive-border-light dark:border-hive-border px-4 py-4 bg-white dark:bg-hive-surface">
        <InputArea />
      </div>
    </div>
  );
};
