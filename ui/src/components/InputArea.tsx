import React, { useRef, useEffect, useState } from "react";
import { useChatStore } from "@/stores/chatStore";
import { sendMessage } from "@/lib/tauri";
import { Send, Paperclip } from "lucide-react";

interface InputAreaProps {
  quickPrompt?: string;
  onQuickPromptUsed?: () => void;
}

export const InputArea: React.FC<InputAreaProps> = ({ quickPrompt = "", onQuickPromptUsed }) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [input, setInput] = useState("");
  const isStreaming = useChatStore((state) => state.isStreaming);
  const userSendMessage = useChatStore((state) => state.sendMessage);
  const setChatStreaming = useChatStore((state) => state.setIsStreaming);
  const appendStreamDelta = useChatStore((state) => state.appendStreamDelta);
  const completeStream = useChatStore((state) => state.completeStream);

  // Handle quick prompt from welcome screen
  useEffect(() => {
    if (quickPrompt) {
      setInput(quickPrompt);
      onQuickPromptUsed?.();
      // Focus textarea for better UX
      setTimeout(() => textareaRef.current?.focus(), 0);
    }
  }, [quickPrompt, onQuickPromptUsed]);

  const autoResizeTextarea = () => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = "auto";
      textarea.style.height = Math.min(textarea.scrollHeight, 200) + "px";
    }
  };

  useEffect(() => {
    autoResizeTextarea();
  }, [input]);

  const handleSendMessage = async () => {
    if (!input.trim() || isStreaming) return;

    const messageContent = input.trim();
    setInput("");

    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }

    userSendMessage(messageContent);
    setChatStreaming(true);

    try {
      const response = await sendMessage(messageContent);
      appendStreamDelta(response);
    } catch (error) {
      appendStreamDelta(
        `Error: ${error instanceof Error ? error.message : "Failed to send message"}`
      );
    } finally {
      completeStream();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.currentTarget.files;
    if (files && files.length > 0) {
      const file = files[0];
      setInput((prev) => `${prev}\n[Attached: ${file.name}]`);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask anything... code, debug, refactor, explain"
          disabled={isStreaming}
          rows={1}
          className="input-base max-h-48 resize-none pl-12 pr-12"
        />

        <button
          onClick={handleSendMessage}
          disabled={!input.trim() || isStreaming}
          className="absolute right-3 bottom-3 p-2 text-hive-cyan hover:text-hive-magenta disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          title="Send message (Ctrl+Enter)"
        >
          <Send className="w-5 h-5" />
        </button>

        <input
          ref={fileInputRef}
          type="file"
          onChange={handleFileInput}
          className="hidden"
        />

        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={isStreaming}
          className="absolute left-3 bottom-3 p-2 text-slate-400 hover:text-hive-cyan disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          title="Attach file"
        >
          <Paperclip className="w-5 h-5" />
        </button>
      </div>

      <div className="flex justify-between items-center text-xs text-slate-500 dark:text-slate-400">
        <div className="flex items-center gap-2">
          <kbd className="px-1.5 py-0.5 rounded bg-slate-200 dark:bg-slate-700 text-[10px] font-mono">
            {navigator.platform?.includes("Mac") ? "⌘" : "Ctrl"}+↵
          </kbd>
          <span>to send</span>
        </div>
        <div>
          {isStreaming && (
            <span className="text-hive-cyan animate-pulse">Generating...</span>
          )}
        </div>
      </div>
    </div>
  );
};
