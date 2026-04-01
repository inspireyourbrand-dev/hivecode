import React, { useEffect, useRef, useState } from "react";
import { useChatStore } from "@/stores/chatStore";
import { useAppStore } from "@/stores/appStore";
import { MessageBubble } from "./MessageBubble";
import { InputArea } from "./InputArea";
import { useAutoScroll } from "@/hooks/useAutoScroll";
import { Message } from "@/lib/types";
import { Code2, FolderSearch, Layers, Settings } from "lucide-react";

export const ChatPanel: React.FC = () => {
  const messages = useChatStore((state) => state.messages);
  const isStreaming = useChatStore((state) => state.isStreaming);
  const currentStreamContent = useChatStore(
    (state) => state.currentStreamContent
  );
  const scrollRef = useAutoScroll(messages.length);
  const containerRef = useRef<HTMLDivElement>(null);
  const [quickPrompt, setQuickPrompt] = useState("");

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

  // Handle quick prompt card clicks
  const handleQuickPrompt = (prompt: string) => {
    setQuickPrompt(prompt);
  };

  // HiveCode Hexagon SVG Logo with glow effect
  const HiveCodeLogo = () => (
    <svg
      width="80"
      height="80"
      viewBox="0 0 80 80"
      className="mx-auto mb-4"
      style={{
        filter: "drop-shadow(0 0 20px rgba(62, 186, 244, 0.6)) drop-shadow(0 0 40px rgba(223, 48, 255, 0.3))",
      }}
    >
      <defs>
        <linearGradient id="hexGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#3ebaf4" />
          <stop offset="100%" stopColor="#df30ff" />
        </linearGradient>
      </defs>

      {/* Outer hexagon */}
      <path
        d="M40 10 L65 22.5 L65 57.5 L40 70 L15 57.5 L15 22.5 Z"
        fill="none"
        stroke="url(#hexGradient)"
        strokeWidth="2"
      />

      {/* Inner "H" shape using hexagon segments */}
      {/* Left vertical line */}
      <line x1="28" y1="28" x2="28" y2="52" stroke="url(#hexGradient)" strokeWidth="2" strokeLinecap="round" />

      {/* Right vertical line */}
      <line x1="52" y1="28" x2="52" y2="52" stroke="url(#hexGradient)" strokeWidth="2" strokeLinecap="round" />

      {/* Horizontal connector */}
      <line x1="28" y1="40" x2="52" y2="40" stroke="url(#hexGradient)" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Messages Container */}
      <div
        ref={containerRef}
        className="flex-1 overflow-y-auto px-4 py-6 space-y-2"
      >
        {messages.length === 0 && !isStreaming ? (
          <div className="h-full flex flex-col items-center justify-center text-center constellation-bg">
            <div className="mb-8">
              {/* Branded Logo */}
              <HiveCodeLogo />

              {/* Title with gradient text */}
              <h2 className="text-4xl font-bold gradient-text mb-2">
                Welcome to HiveCode
              </h2>

              {/* Animated Tagline */}
              <p className="text-slate-600 dark:text-slate-400 max-w-md mx-auto animate-pulse">
                Your elite AI-powered coding companion
              </p>
            </div>

            {/* Quick Prompt Cards Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-8 max-w-2xl w-full px-4">
              {/* Code Generation Card */}
              <div
                onClick={() => handleQuickPrompt("Write me a React component that...")}
                className="group p-4 rounded-lg bg-white/5 dark:bg-hive-surface/50 border border-slate-300/20 dark:border-hive-cyan/30 hover:border-hive-cyan/80 cursor-pointer transition-all duration-300 hover:shadow-lg"
                style={{
                  boxShadow:
                    "inset 0 0 20px rgba(62, 186, 244, 0), 0 0 20px rgba(62, 186, 244, 0)",
                }}
                onMouseEnter={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0.1), 0 0 20px rgba(62, 186, 244, 0.3)";
                }}
                onMouseLeave={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0), 0 0 20px rgba(62, 186, 244, 0)";
                }}
              >
                <div className="flex items-start gap-3">
                  <Code2 className="w-5 h-5 text-hive-cyan flex-shrink-0 mt-0.5" />
                  <div className="text-left">
                    <p className="font-semibold text-slate-900 dark:text-white text-sm group-hover:text-hive-cyan transition-colors">
                      Code Generation
                    </p>
                    <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                      Write functions, components, or snippets
                    </p>
                  </div>
                </div>
              </div>

              {/* Code Analysis Card */}
              <div
                onClick={() => handleQuickPrompt("Analyze this code and explain...")}
                className="group p-4 rounded-lg bg-white/5 dark:bg-hive-surface/50 border border-slate-300/20 dark:border-hive-cyan/30 hover:border-hive-cyan/80 cursor-pointer transition-all duration-300 hover:shadow-lg"
                onMouseEnter={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0.1), 0 0 20px rgba(62, 186, 244, 0.3)";
                }}
                onMouseLeave={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0), 0 0 20px rgba(62, 186, 244, 0)";
                }}
              >
                <div className="flex items-start gap-3">
                  <FolderSearch className="w-5 h-5 text-hive-cyan flex-shrink-0 mt-0.5" />
                  <div className="text-left">
                    <p className="font-semibold text-slate-900 dark:text-white text-sm group-hover:text-hive-cyan transition-colors">
                      Code Analysis
                    </p>
                    <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                      Debug, refactor, or explore code
                    </p>
                  </div>
                </div>
              </div>

              {/* Architecture Card */}
              <div
                onClick={() => handleQuickPrompt("Help me design a system architecture for...")}
                className="group p-4 rounded-lg bg-white/5 dark:bg-hive-surface/50 border border-slate-300/20 dark:border-hive-cyan/30 hover:border-hive-cyan/80 cursor-pointer transition-all duration-300 hover:shadow-lg"
                onMouseEnter={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0.1), 0 0 20px rgba(62, 186, 244, 0.3)";
                }}
                onMouseLeave={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0), 0 0 20px rgba(62, 186, 244, 0)";
                }}
              >
                <div className="flex items-start gap-3">
                  <Layers className="w-5 h-5 text-hive-cyan flex-shrink-0 mt-0.5" />
                  <div className="text-left">
                    <p className="font-semibold text-slate-900 dark:text-white text-sm group-hover:text-hive-cyan transition-colors">
                      Architecture
                    </p>
                    <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                      Design systems and project structure
                    </p>
                  </div>
                </div>
              </div>

              {/* Settings Card */}
              <div
                onClick={() => handleQuickPrompt("Configure my development environment...")}
                className="group p-4 rounded-lg bg-white/5 dark:bg-hive-surface/50 border border-slate-300/20 dark:border-hive-cyan/30 hover:border-hive-cyan/80 cursor-pointer transition-all duration-300 hover:shadow-lg"
                onMouseEnter={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0.1), 0 0 20px rgba(62, 186, 244, 0.3)";
                }}
                onMouseLeave={(e) => {
                  const el = e.currentTarget as HTMLElement;
                  el.style.boxShadow =
                    "inset 0 0 20px rgba(62, 186, 244, 0), 0 0 20px rgba(62, 186, 244, 0)";
                }}
              >
                <div className="flex items-start gap-3">
                  <Settings className="w-5 h-5 text-hive-cyan flex-shrink-0 mt-0.5" />
                  <div className="text-left">
                    <p className="font-semibold text-slate-900 dark:text-white text-sm group-hover:text-hive-cyan transition-colors">
                      Configuration
                    </p>
                    <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">
                      Set up tools, libraries, and providers
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Subtle hint text */}
            <p className="text-xs text-slate-500 dark:text-slate-500 mt-8">
              Click any card above or start typing to begin
            </p>
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
        <InputArea quickPrompt={quickPrompt} onQuickPromptUsed={() => setQuickPrompt("")} />
      </div>
    </div>
  );
};
