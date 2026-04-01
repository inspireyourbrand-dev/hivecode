import React, { useEffect, useState, useCallback } from "react";
import { useAppStore } from "@/stores/appStore";
import { useChatStore } from "@/stores/chatStore";
import { useNotification } from "./NotificationToast";
import { Command, X } from "lucide-react";

interface ShortcutCommand {
  keys: string[];
  label: string;
  description: string;
  handler: () => void;
}

interface ShortcutHint {
  keys: string[];
  label: string;
}

const CommandPalette: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  commands: ShortcutCommand[];
}> = ({ isOpen, onClose, commands }) => {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  const filteredCommands = commands.filter(
    (cmd) =>
      cmd.label.toLowerCase().includes(query.toLowerCase()) ||
      cmd.description.toLowerCase().includes(query.toLowerCase())
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) =>
        Math.min(i + 1, filteredCommands.length - 1)
      );
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filteredCommands[selectedIndex]) {
        filteredCommands[selectedIndex].handler();
        onClose();
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/50 flex items-start justify-center pt-20"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl bg-hive-surface border border-hive-cyan rounded-lg overflow-hidden shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-hive-cyan">
          <Command className="w-5 h-5 text-hive-cyan flex-shrink-0" />
          <input
            autoFocus
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search commands..."
            className="flex-1 bg-transparent text-white placeholder-slate-400 outline-none"
          />
          <button
            onClick={onClose}
            className="p-1 hover:bg-white/10 rounded transition-colors"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        {/* Commands List */}
        <div className="max-h-96 overflow-y-auto">
          {filteredCommands.length === 0 ? (
            <div className="px-4 py-8 text-center text-slate-400">
              No commands found
            </div>
          ) : (
            filteredCommands.map((cmd, idx) => (
              <button
                key={idx}
                onClick={() => {
                  cmd.handler();
                  onClose();
                }}
                className={`w-full px-4 py-3 text-left border-b border-hive-border transition-colors ${
                  idx === selectedIndex
                    ? "bg-hive-cyan/10 border-l-2 border-l-hive-cyan"
                    : "hover:bg-white/5"
                }`}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-white">{cmd.label}</p>
                    <p className="text-xs text-slate-400 mt-1">
                      {cmd.description}
                    </p>
                  </div>
                  <kbd className="ml-4 px-2 py-1 text-xs font-mono bg-hive-surface border border-hive-cyan text-hive-cyan rounded whitespace-nowrap">
                    {cmd.keys.join(" + ")}
                  </kbd>
                </div>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export const KeyboardShortcuts: React.FC = () => {
  const [paletteOpen, setPaletteOpen] = useState(false);
  const notification = useNotification();
  const toggleSidebar = useAppStore((state) => state.toggleSidebar);
  const toggleSettings = useAppStore((state) => state.toggleSettings);
  const clearConversation = useChatStore(
    (state) => state.clearConversation
  );
  const newSession = useChatStore((state) => state.newSession);
  const isStreaming = useChatStore((state) => state.isStreaming);

  const commands: ShortcutCommand[] = [
    {
      keys: ["Ctrl", "Enter"],
      label: "Send Message",
      description: "Send your message to the AI",
      handler: () => {
        // Handled by InputArea component
        notification.info("Send Message", "Focus the input area and press Ctrl+Enter");
      },
    },
    {
      keys: ["Ctrl", "N"],
      label: "New Session",
      description: "Start a new conversation",
      handler: () => {
        newSession();
        notification.success("New Session", "Started a new conversation");
      },
    },
    {
      keys: ["Ctrl", "K"],
      label: "Command Palette",
      description: "Open the command palette",
      handler: () => setPaletteOpen(false),
    },
    {
      keys: ["Ctrl", "/"],
      label: "Toggle Sidebar",
      description: "Show or hide the sidebar",
      handler: () => {
        toggleSidebar();
      },
    },
    {
      keys: ["Ctrl", ","],
      label: "Settings",
      description: "Open settings",
      handler: () => {
        toggleSettings();
        notification.info("Settings", "Opening settings panel");
      },
    },
    {
      keys: ["Ctrl", "L"],
      label: "Clear Conversation",
      description: "Clear all messages in the current session",
      handler: () => {
        clearConversation();
        notification.success("Cleared", "Conversation cleared");
      },
    },
    {
      keys: ["Escape"],
      label: "Close/Cancel",
      description: "Close dialogs or cancel operations",
      handler: () => {
        setPaletteOpen(false);
      },
    },
  ];

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Prevent shortcuts when typing in input fields (but allow Escape)
      const target = e.target as HTMLElement;
      const isInput =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        (target.contentEditable && target.contentEditable !== "false");

      if (isInput && e.key !== "Escape") {
        return;
      }

      // Ctrl+K or Cmd+K - Command Palette
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setPaletteOpen((prev) => !prev);
        return;
      }

      // Ctrl+N or Cmd+N - New Session
      if ((e.ctrlKey || e.metaKey) && e.key === "n") {
        e.preventDefault();
        newSession();
        notification.success("New Session", "Started a new conversation");
        return;
      }

      // Ctrl+/ or Cmd+/ - Toggle Sidebar
      if ((e.ctrlKey || e.metaKey) && e.key === "/") {
        e.preventDefault();
        toggleSidebar();
        return;
      }

      // Ctrl+, or Cmd+, - Settings
      if ((e.ctrlKey || e.metaKey) && e.key === ",") {
        e.preventDefault();
        toggleSettings();
        notification.info("Settings", "Opening settings panel");
        return;
      }

      // Ctrl+L or Cmd+L - Clear Conversation
      if ((e.ctrlKey || e.metaKey) && e.key === "l") {
        e.preventDefault();
        if (!isInput) {
          clearConversation();
          notification.success("Cleared", "Conversation cleared");
        }
        return;
      }

      // Escape - Close palette
      if (e.key === "Escape") {
        setPaletteOpen(false);
      }
    },
    [
      toggleSidebar,
      toggleSettings,
      clearConversation,
      newSession,
      notification,
    ]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  return (
    <CommandPalette
      isOpen={paletteOpen}
      onClose={() => setPaletteOpen(false)}
      commands={commands}
    />
  );
};

export const ShortcutHint: React.FC<{
  keys: string[];
  label: string;
}> = ({ keys, label }) => {
  return (
    <div
      className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 transition-colors"
      title={label}
    >
      {keys.map((key, idx) => (
        <React.Fragment key={idx}>
          {idx > 0 && <span>+</span>}
          <kbd className="px-2 py-1 font-mono bg-hive-bg-light dark:bg-hive-surface border border-hive-border-light dark:border-hive-border rounded text-xs">
            {key}
          </kbd>
        </React.Fragment>
      ))}
    </div>
  );
};
