import React from "react";
import { Message, ContentBlock } from "@/lib/types";
import { Copy, Check, AlertCircle, CheckCircle, GitBranch } from "lucide-react";
import Markdown from "react-markdown";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { atomDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { ThinkingPanel } from "./ThinkingPanel";
import { DiffView } from "./DiffView";

interface MessageBubbleProps {
  message: Message;
  thinking?: string;
  isThinking?: boolean;
  onForkBranch?: () => void;
}

export const MessageBubble: React.FC<MessageBubbleProps> = ({
  message,
  thinking,
  isThinking = false,
  onForkBranch,
}) => {
  const [copiedId, setCopiedId] = React.useState<string | null>(null);
  const [showForkButton, setShowForkButton] = React.useState(false);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const renderTextContent = (text: string) => {
    return (
      <Markdown
        className="markdown text-sm"
        components={{
          code({ className, children, ...props }: any) {
            const match = /language-(\w+)/.exec(className || "");
            const language = match ? match[1] : "";
            const isInline = !match;

            if (isInline) {
              return (
                <code className="bg-hive-border-light dark:bg-hive-border px-1.5 py-0.5 rounded text-hive-magenta dark:text-hive-cyan">
                  {children}
                </code>
              );
            }

            const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
            const codeContent = String(children).replace(/\n$/, "");

            return (
              <div className="relative group my-3 rounded-lg overflow-hidden code-block">
                <button
                  onClick={() => copyToClipboard(codeContent, codeId)}
                  className="copy-button"
                  title="Copy code"
                >
                  {copiedId === codeId ? (
                    <Check className="w-4 h-4 text-green-500" />
                  ) : (
                    <Copy className="w-4 h-4 text-slate-400 hover:text-slate-200" />
                  )}
                </button>
                <div className="absolute top-2 left-2 text-xs text-slate-500 opacity-50">
                  {language}
                </div>
                <SyntaxHighlighter
                  language={language}
                  style={atomDark}
                  customStyle={{
                    margin: 0,
                    padding: "1rem",
                    paddingTop: "2rem",
                    background: "transparent",
                  }}
                >
                  {codeContent}
                </SyntaxHighlighter>
              </div>
            );
          },
          a({ href, children }) {
            return (
              <a
                href={href}
                className="text-hive-cyan hover:text-hive-magenta underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                {children}
              </a>
            );
          },
          h1({ children }) {
            return <h1 className="text-2xl font-bold mt-4 mb-2">{children}</h1>;
          },
          h2({ children }) {
            return <h2 className="text-xl font-bold mt-4 mb-2">{children}</h2>;
          },
          h3({ children }) {
            return <h3 className="text-lg font-bold mt-3 mb-2">{children}</h3>;
          },
          h4({ children }) {
            return <h4 className="text-base font-bold mt-3 mb-2">{children}</h4>;
          },
          ul({ children }) {
            return <ul className="list-disc list-inside ml-4 mb-3">{children}</ul>;
          },
          ol({ children }) {
            return <ol className="list-decimal list-inside ml-4 mb-3">{children}</ol>;
          },
          li({ children }) {
            return <li className="mb-1">{children}</li>;
          },
          blockquote({ children }) {
            return (
              <blockquote className="border-l-4 border-hive-cyan pl-4 italic my-3 text-slate-600 dark:text-slate-400">
                {children}
              </blockquote>
            );
          },
          table({ children }) {
            return (
              <table className="w-full border-collapse my-3 border border-hive-border-light dark:border-hive-border">
                {children}
              </table>
            );
          },
          thead({ children }) {
            return (
              <thead className="bg-hive-bg-light dark:bg-hive-surface">
                {children}
              </thead>
            );
          },
          th({ children }) {
            return (
              <th className="border border-hive-border-light dark:border-hive-border p-2 text-left font-semibold">
                {children}
              </th>
            );
          },
          td({ children }) {
            return (
              <td className="border border-hive-border-light dark:border-hive-border p-2">
                {children}
              </td>
            );
          },
        }}
      >
        {text}
      </Markdown>
    );
  };

  const renderContent = (blocks: ContentBlock[]) => {
    return blocks.map((block, idx) => {
      switch (block.type) {
        case "text":
          return (
            <div key={idx} className="w-full">
              {renderTextContent(block.text || "")}
            </div>
          );

        case "tool_use":
          return (
            <div key={idx} className="tool-card running my-2 w-full">
              <div className="flex items-center gap-2 mb-2">
                <div className="spinner text-hive-cyan" />
                <span className="font-semibold text-sm">
                  {block.tool_use?.name}
                </span>
              </div>
              <div className="text-xs text-slate-600 dark:text-slate-400">
                <pre className="overflow-x-auto bg-hive-bg-light dark:bg-slate-900 p-2 rounded">
                  {JSON.stringify(block.tool_use?.input, null, 2)}
                </pre>
              </div>
            </div>
          );

        case "tool_result":
          const isError = block.tool_result?.is_error || false;
          return (
            <div
              key={idx}
              className={`tool-card ${isError ? "error" : "success"} my-2 w-full`}
            >
              <div className="flex items-center gap-2 mb-2">
                {isError ? (
                  <AlertCircle className="w-4 h-4 text-red-500" />
                ) : (
                  <CheckCircle className="w-4 h-4 text-green-500" />
                )}
                <span className="font-semibold text-sm">
                  Tool Result
                </span>
              </div>
              <div className="text-xs text-slate-600 dark:text-slate-400">
                <pre className="overflow-x-auto bg-hive-bg-light dark:bg-slate-900 p-2 rounded">
                  {block.tool_result?.content}
                </pre>
              </div>
            </div>
          );

        default:
          return null;
      }
    });
  };

  const baseClasses =
    message.role === "user"
      ? "message-bubble-user"
      : message.role === "system"
        ? "message-bubble-system"
        : "message-bubble-assistant";

  const containerClass =
    message.role === "user" ? "justify-end" : "justify-start";

  return (
    <div
      className={`flex ${containerClass} mb-4 animate-slide-in group`}
      onMouseEnter={() => message.role === "assistant" && setShowForkButton(true)}
      onMouseLeave={() => setShowForkButton(false)}
    >
      <div className={baseClasses}>
        {message.role === "assistant" && thinking && (
          <ThinkingPanel
            thinking={thinking}
            isStreaming={isThinking}
            thinkingType="reasoning"
          />
        )}
        {renderContent(message.content)}
        <div className="text-xs opacity-70 mt-2 flex items-center justify-between">
          <span>
            {new Date(message.timestamp).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </span>
          {message.role === "assistant" && showForkButton && (
            <button
              onClick={onForkBranch}
              className="opacity-0 group-hover:opacity-100 transition-opacity p-1.5 rounded hover:bg-hive-border text-hive-magenta"
              title="Fork conversation here"
            >
              <GitBranch className="w-3 h-3" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
