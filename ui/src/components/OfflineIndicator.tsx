import React, { useState, useEffect } from "react";
import { Circle, Wifi, WifiOff, AlertCircle } from "lucide-react";

interface OfflineIndicatorProps {
  isOnline: boolean;
  isDegraded?: boolean;
  usingLocalModel?: boolean;
  lastCheckTime?: string;
  onForceCheck?: () => void;
}

export const OfflineIndicator: React.FC<OfflineIndicatorProps> = ({
  isOnline,
  isDegraded = false,
  usingLocalModel = false,
  lastCheckTime,
  onForceCheck,
}) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const [isChecking, setIsChecking] = useState(false);

  const handleForceCheck = async () => {
    setIsChecking(true);
    try {
      await onForceCheck?.();
    } finally {
      setTimeout(() => setIsChecking(false), 1000);
    }
  };

  const getStatusColor = () => {
    if (isDegraded) return "text-hive-yellow";
    if (!isOnline) return "text-red-500";
    return "text-hive-green";
  };

  const getStatusLabel = () => {
    if (!isOnline) return "Offline";
    if (isDegraded) return "Degraded";
    return "Online";
  };

  const getStatusMessage = () => {
    if (!isOnline) {
      return "Running in offline mode. Requests will be queued and sent when connection is restored.";
    }
    if (isDegraded) {
      return "Connection is degraded. Some API calls may be slow or fail.";
    }
    return "Connected to all services.";
  };

  return (
    <div className="relative">
      <button
        onClick={() => setShowTooltip(!showTooltip)}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
        className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-hive-surface border border-hive-border hover:bg-hive-border/50 transition-colors group"
        title={getStatusLabel()}
      >
        <div className="relative">
          <Circle
            className={`w-3 h-3 ${getStatusColor()} transition-colors ${
              isChecking ? "animate-pulse" : ""
            }`}
            fill="currentColor"
          />
        </div>

        {usingLocalModel && (
          <span className="text-xs px-2 py-0.5 rounded bg-hive-magenta/20 text-hive-magenta font-medium">
            Local model
          </span>
        )}

        {!isOnline && (
          <WifiOff className="w-3 h-3 text-red-500 opacity-0 group-hover:opacity-100 transition-opacity" />
        )}

        {isDegraded && (
          <AlertCircle className="w-3 h-3 text-hive-yellow opacity-0 group-hover:opacity-100 transition-opacity" />
        )}
      </button>

      {/* Tooltip */}
      {showTooltip && (
        <div className="absolute right-0 mt-2 w-64 bg-hive-bg border border-hive-border rounded-lg shadow-lg z-50 p-3">
          <div className="mb-2">
            <div className="flex items-center gap-2 mb-1">
              <Circle
                className={`w-3 h-3 ${getStatusColor()}`}
                fill="currentColor"
              />
              <span className="text-sm font-semibold text-white">
                {getStatusLabel()}
              </span>
            </div>
            <p className="text-xs text-slate-400">{getStatusMessage()}</p>
          </div>

          {lastCheckTime && (
            <p className="text-xs text-slate-500 mb-2">
              Last check: {new Date(lastCheckTime).toLocaleTimeString()}
            </p>
          )}

          {usingLocalModel && (
            <div className="mb-2 p-2 rounded bg-hive-magenta/10 border border-hive-magenta/30">
              <p className="text-xs text-hive-magenta">
                Using local model for fallback. API calls will fail until connection is restored.
              </p>
            </div>
          )}

          <button
            onClick={handleForceCheck}
            disabled={isChecking}
            className="w-full px-2 py-1 rounded text-xs bg-hive-cyan text-black font-medium hover:bg-hive-cyan/80 disabled:opacity-50 transition-colors"
          >
            {isChecking ? "Checking..." : "Force recheck"}
          </button>
        </div>
      )}
    </div>
  );
};
