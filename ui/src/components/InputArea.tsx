import React, { useRef, useEffect, useState } from "react";
import { useChatStore } from "@/stores/chatStore";
import { useAppStore } from "@/stores/appStore";
import { sendMessage } from "@/lib/tauri";
import { Send, Paperclip } from "lucide-react";

export const InputArea: React.FC = () => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [input, setInput] = useState("");
  const isStreaming = useChatStore((state) => state.isStreaming);
  const userSendMessage = useChatStore((state) => state.sendMessage);
  const setChatStreaming = useChatStore((state) => state.setIsStreaming);
  const appendStreamDelta = useChatStore((state) => state.appendStreamDelta);
  const completeStream = useChatStore((state) => state.completeStream);

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

  const characterCount = input.length;
  const lineCount = input.split("\n").length;

  return (
    <div className="flex flex-col gap-3">
      <div className="relative">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask me to write code, fix bugs, or explain concepts... (Ctrl+Enter to send)"
          disabled={isStreaming}
          rows={1}
          className="input-base max-h-48 resize-none pr-12"
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
        <div>
          {characterCount} characters • {lineCount} line{lineCount !== 1 ? "s" : ""}
        </div>
        <div>
          {isStreaming && (
            <span className="text-hive-cyan">Receiving response...</span>
          )}
        </div>
      </div>
    </div>
  );
};
