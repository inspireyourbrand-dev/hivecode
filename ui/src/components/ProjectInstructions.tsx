import React, { useState, useEffect } from "react";
import { FileText, Save, Edit2, AlertCircle, CheckCircle } from "lucide-react";

interface ProjectInstructionsProps {
  content: string;
  onSave?: (content: string) => Promise<void>;
  onLoadTemplate?: () => Promise<string>;
}

export const ProjectInstructions: React.FC<ProjectInstructionsProps> = ({
  content,
  onSave,
  onLoadTemplate,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editedContent, setEditedContent] = useState(content);
  const [isSaving, setIsSaving] = useState(false);
  const [validation, setValidation] = useState<{
    isValid: boolean;
    issues: string[];
  }>({ isValid: true, issues: [] });

  useEffect(() => {
    setEditedContent(content);
  }, [content]);

  const validateContent = (text: string) => {
    const issues: string[] = [];

    if (!text.trim()) {
      issues.push("Instructions cannot be empty");
    }

    const hasInstructions = text.includes("## Instructions");
    const hasTools = text.includes("## Tools");
    const hasFiles = text.includes("## Files");

    if (!hasInstructions) {
      issues.push("Missing ## Instructions section");
    }
    if (!hasTools) {
      issues.push("Missing ## Tools section");
    }

    setValidation({
      isValid: issues.length === 0,
      issues,
    });
  };

  const handleSave = async () => {
    validateContent(editedContent);
    if (validation.isValid || validation.issues.length === 0) {
      setIsSaving(true);
      try {
        await onSave?.(editedContent);
        setIsEditing(false);
      } finally {
        setIsSaving(false);
      }
    }
  };

  const handleLoadTemplate = async () => {
    const template = await onLoadTemplate?.();
    if (template) {
      setEditedContent(template);
      validateContent(template);
      setIsEditing(true);
    }
  };

  return (
    <div className="rounded-lg border border-hive-border bg-hive-surface h-full flex flex-col">
      {/* Header */}
      <div className="px-4 py-3 border-b border-hive-border/50 flex items-center justify-between flex-shrink-0">
        <h3 className="text-sm font-semibold text-white flex items-center gap-2">
          <FileText className="w-4 h-4 text-hive-cyan" />
          HIVECODE.md
        </h3>
        <div className="flex items-center gap-2">
          {!isEditing && (
            <>
              <button
                onClick={() => setIsEditing(true)}
                className="p-1.5 rounded hover:bg-hive-border transition-colors text-hive-magenta"
                title="Edit"
              >
                <Edit2 className="w-4 h-4" />
              </button>
              <button
                onClick={handleLoadTemplate}
                className="text-xs px-3 py-1.5 rounded bg-hive-border hover:bg-hive-border/80 text-slate-300 transition-colors"
              >
                Template
              </button>
            </>
          )}
        </div>
      </div>

      {/* Validation Info */}
      {isEditing && validation.issues.length > 0 && (
        <div className="px-4 py-2 bg-yellow-950/50 border-b border-hive-yellow/30">
          {validation.issues.map((issue, idx) => (
            <div key={idx} className="flex items-center gap-2 text-xs text-yellow-200">
              <AlertCircle className="w-3 h-3" />
              {issue}
            </div>
          ))}
        </div>
      )}

      {/* Content Area */}
      <div className="flex-1 overflow-hidden flex flex-col">
        {isEditing ? (
          <textarea
            value={editedContent}
            onChange={(e) => {
              setEditedContent(e.target.value);
              validateContent(e.target.value);
            }}
            className="flex-1 px-4 py-3 bg-hive-bg text-white font-mono text-xs resize-none focus:outline-none border-0"
            placeholder="# HiveCode Instructions

## Instructions
Describe the project context and goals here.

## Tools
List allowed tools here.

## Files
Specify file restrictions here.

## Model Preferences
Describe model preferences here."
          />
        ) : (
          <div className="flex-1 overflow-y-auto px-4 py-3">
            <pre className="text-xs text-slate-300 whitespace-pre-wrap break-words font-mono">
              {content || "(No instructions set)"}
            </pre>
          </div>
        )}
      </div>

      {/* Footer */}
      {isEditing && (
        <div className="px-4 py-3 border-t border-hive-border/50 flex items-center justify-end gap-2 flex-shrink-0">
          <button
            onClick={() => {
              setIsEditing(false);
              setEditedContent(content);
            }}
            className="px-3 py-1.5 rounded text-sm bg-hive-border hover:bg-hive-border/80 text-slate-300 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving || !validation.isValid}
            className="px-3 py-1.5 rounded text-sm bg-hive-cyan text-black font-medium hover:bg-hive-cyan/80 disabled:opacity-50 flex items-center gap-2 transition-colors"
          >
            <Save className="w-4 h-4" />
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      )}

      {/* Read-only Footer */}
      {!isEditing && (
        <div className="px-4 py-2 border-t border-hive-border/50 flex items-center gap-2 text-xs text-slate-500 flex-shrink-0">
          {validation.isValid ? (
            <>
              <CheckCircle className="w-3 h-3 text-hive-green" />
              Valid configuration
            </>
          ) : (
            <>
              <AlertCircle className="w-3 h-3 text-hive-yellow" />
              {validation.issues.length} issues
            </>
          )}
        </div>
      )}
    </div>
  );
};
