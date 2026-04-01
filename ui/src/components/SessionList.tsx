import React, { useEffect, useState, useCallback } from "react";
import { useChatStore, SessionSummary } from "@/stores/chatStore";
import { Trash2, Plus, Search, Clock, Zap } from "lucide-react";

interface SessionListProps {
  onSessionSelect?: (sessionId: string) => void;
  className?: string;
}

export const SessionList: React.FC<SessionListProps> = ({
  onSessionSelect,
  className = "",
}) => {
  const {
    sessions,
    currentSessionId,
    refreshSessions,
    loadSession,
    deleteSession,
    newSession,
    searchSessions,
  } = useChatStore();

  const [searchQuery, setSearchQuery] = useState("");
  const [filteredSessions, setFilteredSessions] = useState<SessionSummary[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null);

  // Load sessions on mount
  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  // Update filtered sessions when sessions or search query changes
  useEffect(() => {
    if (searchQuery.trim() === "") {
      setFilteredSessions(sessions);
    } else {
      // Filter locally for instant UI feedback
      const query = searchQuery.toLowerCase();
      setFilteredSessions(
        sessions.filter(
          (session) =>
            session.title.toLowerCase().includes(query) ||
            session.model_used.toLowerCase().includes(query)
        )
      );
    }
  }, [sessions, searchQuery]);

  const handleSearch = useCallback(
    async (query: string) => {
      setSearchQuery(query);
      if (query.trim().length > 0) {
        setIsLoading(true);
        try {
          const results = await searchSessions(query);
          setFilteredSessions(results);
        } finally {
          setIsLoading(false);
        }
      }
    },
    [searchSessions]
  );

  const handleLoadSession = async (sessionId: string) => {
    await loadSession(sessionId);
    onSessionSelect?.(sessionId);
  };

  const handleNewSession = async () => {
    await newSession();
  };

  const handleDeleteSession = async (sessionId: string) => {
    await deleteSession(sessionId);
    setShowDeleteConfirm(null);

    // If deleted session was current, clear selection
    if (currentSessionId === sessionId) {
      setShowDeleteConfirm(null);
    }
  };

  const formatDate = (dateString: string): string => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 60) {
      return `${diffMins}m ago`;
    } else if (diffHours < 24) {
      return `${diffHours}h ago`;
    } else if (diffDays < 7) {
      return `${diffDays}d ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  return (
    <div
      className={`flex flex-col h-full bg-hive-bg-light dark:bg-hive-surface rounded-lg border border-hive-border-light dark:border-hive-border overflow-hidden ${className}`}
    >
      {/* Header */}
      <div className="p-4 border-b border-hive-border-light dark:border-hive-border">
        <h2 className="text-lg font-semibold text-slate-900 dark:text-white mb-3">
          Chat History
        </h2>

        {/* New Chat Button */}
        <button
          onClick={handleNewSession}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-gradient-to-r from-hive-cyan to-hive-magenta hover:shadow-lg text-white font-medium rounded-lg transition-all mb-3"
        >
          <Plus size={16} />
          New Chat
        </button>

        {/* Search Input */}
        <div className="relative">
          <Search
            size={16}
            className="absolute left-3 top-3 text-slate-400 dark:text-slate-500 pointer-events-none"
          />
          <input
            type="text"
            placeholder="Search sessions..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            className="input-base pl-9 text-sm"
          />
        </div>
      </div>

      {/* Sessions List */}
      <div className="flex-1 overflow-y-auto">
        {filteredSessions.length === 0 ? (
          <div className="flex items-center justify-center h-full text-slate-400 dark:text-slate-500 text-sm p-4 text-center">
            {searchQuery.trim() === "" ? (
              <div>
                <p className="mb-2">No chat history yet</p>
                <p className="text-xs opacity-75">
                  Start a conversation to create a session
                </p>
              </div>
            ) : (
              <p>No sessions match "{searchQuery}"</p>
            )}
          </div>
        ) : (
          <div className="divide-y divide-hive-border-light dark:divide-hive-border">
            {filteredSessions.map((session) => (
              <div
                key={session.id}
                className={`group relative border-l-2 transition-all ${
                  currentSessionId === session.id
                    ? "border-l-hive-cyan bg-hive-border-light dark:bg-hive-border"
                    : "border-l-transparent hover:border-l-hive-magenta hover:bg-hive-border-light dark:hover:bg-hive-border"
                }`}
              >
                {/* Delete Confirmation */}
                {showDeleteConfirm === session.id && (
                  <div className="absolute inset-0 bg-red-900/90 backdrop-blur-sm flex items-center justify-center z-50 rounded">
                    <div className="flex flex-col gap-2">
                      <p className="text-sm font-medium text-white">
                        Delete this chat?
                      </p>
                      <div className="flex gap-2">
                        <button
                          onClick={() => handleDeleteSession(session.id)}
                          className="flex-1 px-2 py-1 bg-red-600 hover:bg-red-700 text-white text-xs font-medium rounded transition-colors"
                        >
                          Delete
                        </button>
                        <button
                          onClick={() => setShowDeleteConfirm(null)}
                          className="flex-1 px-2 py-1 bg-slate-600 hover:bg-slate-700 text-white text-xs font-medium rounded transition-colors"
                        >
                          Cancel
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                {/* Session Item */}
                <div
                  onClick={() => handleLoadSession(session.id)}
                  className="p-3 cursor-pointer"
                >
                  {/* Title */}
                  <p className="font-medium text-slate-900 dark:text-white truncate text-sm mb-1">
                    {session.title}
                  </p>

                  {/* Metadata */}
                  <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 mb-2">
                    <Clock size={12} />
                    <span>{formatDate(session.updated_at)}</span>
                    <span className="opacity-50">•</span>
                    <Zap size={12} />
                    <span>{session.model_used}</span>
                    <span className="opacity-50">•</span>
                    <span>{session.message_count} msg</span>
                  </div>

                  {/* Token Count */}
                  <div className="text-xs text-slate-400 dark:text-slate-500">
                    {session.token_count} tokens
                  </div>
                </div>

                {/* Delete Button - appears on hover */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowDeleteConfirm(session.id);
                  }}
                  className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity p-1.5 hover:bg-red-500/20 hover:text-red-400 text-slate-400 dark:text-slate-500 rounded transition-colors"
                  title="Delete session"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer - Session Count */}
      {sessions.length > 0 && (
        <div className="p-3 border-t border-hive-border-light dark:border-hive-border text-xs text-slate-500 dark:text-slate-400 text-center">
          {sessions.length} session{sessions.length !== 1 ? "s" : ""}
        </div>
      )}
    </div>
  );
};

export default SessionList;
