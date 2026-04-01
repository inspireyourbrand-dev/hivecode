import React, { useState, useMemo } from "react";
import { useNotification } from "./NotificationToast";
import {
  Plus,
  Trash2,
  Edit2,
  X,
  Search,
  ChevronDown,
  ChevronUp,
} from "lucide-react";

interface Memory {
  id: string;
  category: string;
  content: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

interface MemoryPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export const MemoryPanel: React.FC<MemoryPanelProps> = ({
  isOpen,
  onClose,
}) => {
  const [memories, setMemories] = useState<Memory[]>([
    {
      id: "m1",
      category: "preferences",
      content: "User prefers TypeScript with strict mode enabled",
      tags: ["coding", "preferences"],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
    {
      id: "m2",
      category: "project",
      content: "HiveCode is a Rust+Tauri v2 AI coding assistant",
      tags: ["hivecode", "project"],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
  ]);
  const [searchQuery, setSearchQuery] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set(["preferences", "project"])
  );
  const [newMemory, setNewMemory] = useState({
    category: "preferences",
    content: "",
    tags: "",
  });
  const notification = useNotification();

  const categories = useMemo(
    () => Array.from(new Set(memories.map((m) => m.category))),
    [memories]
  );

  const filteredMemories = useMemo(() => {
    const query = searchQuery.toLowerCase();
    return memories.filter(
      (m) =>
        m.content.toLowerCase().includes(query) ||
        m.tags.some((tag) => tag.toLowerCase().includes(query)) ||
        m.category.toLowerCase().includes(query)
    );
  }, [memories, searchQuery]);

  const memoriesByCategory = useMemo(
    () =>
      categories.reduce(
        (acc, cat) => {
          acc[cat] = filteredMemories.filter((m) => m.category === cat);
          return acc;
        },
        {} as Record<string, Memory[]>
      ),
    [categories, filteredMemories]
  );

  const handleAddMemory = () => {
    if (!newMemory.content.trim()) {
      notification.warning("Required", "Please enter memory content");
      return;
    }

    const memory: Memory = {
      id: `m${Date.now()}`,
      category: newMemory.category,
      content: newMemory.content,
      tags: newMemory.tags
        .split(",")
        .map((t) => t.trim())
        .filter((t) => t),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    setMemories([...memories, memory]);
    setNewMemory({ category: "preferences", content: "", tags: "" });
    setShowAddForm(false);
    notification.success("Memory Saved", "New memory added");
  };

  const handleDeleteMemory = (id: string) => {
    setMemories(memories.filter((m) => m.id !== id));
    notification.success("Memory Deleted", "Memory removed");
  };

  const toggleCategory = (category: string) => {
    const newExpanded = new Set(expandedCategories);
    if (newExpanded.has(category)) {
      newExpanded.delete(category);
    } else {
      newExpanded.add(category);
    }
    setExpandedCategories(newExpanded);
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-40 bg-black/50"
      onClick={onClose}
    >
      <div
        className="absolute right-0 top-0 bottom-0 w-full max-w-md bg-hive-surface border-l border-hive-border flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-hive-border">
          <h2 className="text-lg font-semibold text-white">Memories</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-white/10 rounded transition-colors"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        {/* Search */}
        <div className="p-4 border-b border-hive-border">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-slate-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search memories..."
              className="input-base pl-10"
            />
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {Object.entries(memoriesByCategory).map(([category, catMemories]) => (
            <div key={category} className="border border-hive-border rounded-lg overflow-hidden">
              {/* Category Header */}
              <button
                onClick={() => toggleCategory(category)}
                className="w-full flex items-center justify-between p-3 bg-hive-bg hover:bg-hive-bg/80 transition-colors"
              >
                <span className="text-sm font-semibold text-white capitalize">
                  {category}
                </span>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-slate-400">
                    {catMemories.length}
                  </span>
                  {expandedCategories.has(category) ? (
                    <ChevronUp className="w-4 h-4 text-slate-400" />
                  ) : (
                    <ChevronDown className="w-4 h-4 text-slate-400" />
                  )}
                </div>
              </button>

              {/* Category Items */}
              {expandedCategories.has(category) && (
                <div className="divide-y divide-hive-border bg-hive-surface/50">
                  {catMemories.map((memory) => (
                    <div
                      key={memory.id}
                      className="p-3 hover:bg-white/5 transition-colors"
                    >
                      <p className="text-sm text-white mb-2 break-words">
                        {memory.content}
                      </p>
                      {memory.tags.length > 0 && (
                        <div className="flex flex-wrap gap-1 mb-2">
                          {memory.tags.map((tag) => (
                            <span
                              key={tag}
                              className="text-xs px-2 py-0.5 rounded-full bg-hive-cyan/10 text-hive-cyan border border-hive-cyan/30"
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      )}
                      <div className="flex items-center justify-between">
                        <span className="text-xs text-slate-500">
                          {new Date(memory.created_at).toLocaleDateString()}
                        </span>
                        <div className="flex gap-2">
                          <button
                            onClick={() => setEditingId(memory.id)}
                            className="p-1 hover:bg-hive-cyan/10 rounded transition-colors"
                            title="Edit"
                          >
                            <Edit2 className="w-3 h-3 text-hive-cyan" />
                          </button>
                          <button
                            onClick={() => handleDeleteMemory(memory.id)}
                            className="p-1 hover:bg-red-500/10 rounded transition-colors"
                            title="Delete"
                          >
                            <Trash2 className="w-3 h-3 text-red-500" />
                          </button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}

          {memories.length === 0 && (
            <div className="text-center py-8">
              <p className="text-sm text-slate-400 mb-3">
                No memories yet
              </p>
              <p className="text-xs text-slate-500">
                Add memories to personalize your experience
              </p>
            </div>
          )}
        </div>

        {/* Add Memory Button / Form */}
        <div className="border-t border-hive-border p-4 space-y-3">
          {!showAddForm ? (
            <button
              onClick={() => setShowAddForm(true)}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-gradient-to-r from-hive-cyan to-hive-magenta text-white rounded-lg font-medium hover:shadow-lg hover:shadow-hive-cyan/50 transition-all"
            >
              <Plus className="w-4 h-4" />
              Add Memory
            </button>
          ) : (
            <div className="space-y-3 p-3 rounded-lg bg-hive-bg border border-hive-border">
              <div>
                <label className="text-xs font-semibold text-slate-400 mb-1 block">
                  Category
                </label>
                <select
                  value={newMemory.category}
                  onChange={(e) =>
                    setNewMemory({ ...newMemory, category: e.target.value })
                  }
                  className="input-base text-sm"
                >
                  <option value="preferences">Preferences</option>
                  <option value="project">Project</option>
                  <option value="context">Context</option>
                  <option value="notes">Notes</option>
                </select>
              </div>

              <div>
                <label className="text-xs font-semibold text-slate-400 mb-1 block">
                  Content
                </label>
                <textarea
                  value={newMemory.content}
                  onChange={(e) =>
                    setNewMemory({ ...newMemory, content: e.target.value })
                  }
                  placeholder="Enter memory content..."
                  rows={3}
                  className="input-base text-sm resize-none"
                />
              </div>

              <div>
                <label className="text-xs font-semibold text-slate-400 mb-1 block">
                  Tags (comma-separated)
                </label>
                <input
                  type="text"
                  value={newMemory.tags}
                  onChange={(e) =>
                    setNewMemory({ ...newMemory, tags: e.target.value })
                  }
                  placeholder="e.g., coding, important"
                  className="input-base text-sm"
                />
              </div>

              <div className="flex gap-2">
                <button
                  onClick={handleAddMemory}
                  className="flex-1 px-3 py-2 bg-hive-cyan text-hive-bg rounded-lg font-medium text-sm hover:bg-hive-cyan/80 transition-colors"
                >
                  Save
                </button>
                <button
                  onClick={() => setShowAddForm(false)}
                  className="flex-1 px-3 py-2 border border-hive-border hover:bg-white/5 rounded-lg font-medium text-sm transition-colors"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
