import React, { useState } from "react";
import { Play, Pause, Square, Volume2, Download, List } from "lucide-react";

interface SessionEvent {
  id: string;
  type: "message" | "tool_use" | "tool_result" | "error";
  timestamp: string;
  data: Record<string, unknown>;
}

interface SessionReplayPlayerProps {
  events: SessionEvent[];
  onExport?: (format: "markdown" | "json") => void;
}

export const SessionReplayPlayer: React.FC<SessionReplayPlayerProps> = ({
  events,
  onExport,
}) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [speed, setSpeed] = useState(1);
  const [showEventList, setShowEventList] = useState(true);

  const totalDuration = events.length > 0 ? events.length * 100 : 0;
  const progress = (currentIndex / Math.max(events.length, 1)) * 100;
  const currentEvent = events[currentIndex];

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleStop = () => {
    setIsPlaying(false);
    setCurrentIndex(0);
  };

  const handleSeek = (index: number) => {
    setCurrentIndex(Math.max(0, Math.min(index, events.length - 1)));
  };

  const formatEventType = (type: string) => {
    return type
      .split("_")
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(" ");
  };

  return (
    <div className="rounded-lg border border-hive-border bg-hive-surface h-full flex flex-col">
      {/* Header */}
      <div className="px-4 py-3 border-b border-hive-border/50 flex items-center justify-between flex-shrink-0">
        <h3 className="text-sm font-semibold text-white">Session Replay</h3>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowEventList(!showEventList)}
            className="p-1.5 rounded hover:bg-hive-border transition-colors text-hive-cyan"
            title="Toggle event list"
          >
            <List className="w-4 h-4" />
          </button>
          <button
            onClick={() => onExport?.("markdown")}
            className="p-1.5 rounded hover:bg-hive-border transition-colors text-slate-400 hover:text-hive-green"
            title="Export as Markdown"
          >
            <Download className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden gap-4 p-4">
        {/* Event List */}
        {showEventList && (
          <div className="w-40 flex-shrink-0 border border-hive-border/50 rounded-lg overflow-hidden flex flex-col bg-hive-bg">
            <div className="px-3 py-2 border-b border-hive-border/50 bg-hive-surface">
              <h4 className="text-xs font-semibold text-slate-300">Events</h4>
            </div>
            <div className="flex-1 overflow-y-auto">
              {events.map((event, idx) => (
                <button
                  key={event.id}
                  onClick={() => handleSeek(idx)}
                  className={`w-full text-left px-3 py-2 text-xs border-b border-hive-border/30 transition-colors ${
                    idx === currentIndex
                      ? "bg-hive-cyan/20 text-hive-cyan border-l-2 border-l-hive-cyan"
                      : "text-slate-400 hover:bg-hive-border/30 hover:text-slate-300"
                  }`}
                >
                  <div className="font-mono text-xs">
                    {formatEventType(event.type)}
                  </div>
                  <div className="text-xs opacity-70">
                    {new Date(event.timestamp).toLocaleTimeString()}
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Main Player */}
        <div className="flex-1 flex flex-col">
          {/* Event Display */}
          <div className="flex-1 mb-4 p-3 rounded-lg bg-hive-bg border border-hive-border/50 overflow-y-auto">
            {currentEvent ? (
              <div>
                <div className="flex items-center gap-2 mb-3 pb-3 border-b border-hive-border/50">
                  <div className="w-2 h-2 rounded-full bg-hive-cyan" />
                  <span className="text-sm font-semibold text-white">
                    {formatEventType(currentEvent.type)}
                  </span>
                  <span className="text-xs text-slate-500 ml-auto">
                    {new Date(currentEvent.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <pre className="text-xs text-slate-300 font-mono whitespace-pre-wrap break-words overflow-hidden">
                  {JSON.stringify(currentEvent.data, null, 2)}
                </pre>
              </div>
            ) : (
              <div className="flex items-center justify-center h-full">
                <p className="text-sm text-slate-500">No events loaded</p>
              </div>
            )}
          </div>

          {/* Progress Bar */}
          <div className="mb-4">
            <input
              type="range"
              min="0"
              max={events.length - 1}
              value={currentIndex}
              onChange={(e) => handleSeek(parseInt(e.target.value))}
              className="w-full h-2 bg-hive-border rounded-full appearance-none cursor-pointer accent-hive-cyan"
            />
            <div className="flex justify-between text-xs text-slate-500 mt-1">
              <span>{currentIndex + 1}</span>
              <span>{events.length}</span>
            </div>
          </div>

          {/* Controls */}
          <div className="flex items-center gap-3 px-3 py-3 rounded-lg bg-hive-bg border border-hive-border/50">
            <button
              onClick={handlePlayPause}
              className="flex items-center gap-2 px-3 py-2 rounded bg-hive-cyan text-black font-medium hover:bg-hive-cyan/80 transition-colors"
            >
              {isPlaying ? (
                <>
                  <Pause className="w-4 h-4" />
                  Pause
                </>
              ) : (
                <>
                  <Play className="w-4 h-4" />
                  Play
                </>
              )}
            </button>

            <button
              onClick={handleStop}
              className="p-2 rounded hover:bg-hive-border transition-colors text-slate-400 hover:text-slate-300"
            >
              <Square className="w-4 h-4" />
            </button>

            <div className="flex-1" />

            <div className="flex items-center gap-2">
              <Volume2 className="w-4 h-4 text-slate-400" />
              <select
                value={speed}
                onChange={(e) => setSpeed(parseFloat(e.target.value))}
                className="px-2 py-1 text-xs rounded bg-hive-border border-0 text-slate-300 focus:outline-none"
              >
                <option value={0.5}>0.5x</option>
                <option value={1}>1x</option>
                <option value={2}>2x</option>
                <option value={4}>4x</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
