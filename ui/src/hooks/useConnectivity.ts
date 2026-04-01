import { useEffect, useState, useCallback, useRef } from "react";
import { useAppStore } from "@/stores/appStore";
import { getOfflineStatus, forceConnectivityCheck } from "@/lib/tauri";

interface ConnectivityState {
  isOnline: boolean;
  isDegraded: boolean;
  forceCheck: () => Promise<void>;
}

/**
 * Hook that monitors online/offline connectivity and updates app state.
 *
 * Features:
 * - Listens to window online/offline events
 * - Checks navigator.onLine on mount for initial state
 * - Tries to get detailed offline status from backend (with fallback)
 * - Periodically pings every 30 seconds to detect degraded connections
 * - Updates app store when connectivity changes
 * - Manual force check via forceCheck callback
 *
 * Returns the current connectivity state for convenience.
 */
export function useConnectivity(): ConnectivityState {
  const [isOnline, setIsOnline] = useState(navigator.onLine);
  const [isDegraded, setIsDegraded] = useState(false);

  const setOnlineStatus = useAppStore((state) => state.setOnlineStatus);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const initializationRef = useRef(false);

  /**
   * Update store with current connectivity state
   */
  const updateStore = useCallback(
    (online: boolean, degraded: boolean) => {
      setOnlineStatus(online, degraded, false);
    },
    [setOnlineStatus]
  );

  /**
   * Check connectivity with backend and update state
   */
  const checkConnectivity = useCallback(async () => {
    try {
      // Try to get detailed offline status from backend
      const offlineStatus = await getOfflineStatus();

      setIsOnline(offlineStatus.isOnline);
      setIsDegraded(offlineStatus.isDegraded);
      updateStore(offlineStatus.isOnline, offlineStatus.isDegraded);
    } catch (error) {
      // Backend may not be ready, fall back to navigator.onLine
      console.debug(
        "Failed to get offline status from backend, using navigator.onLine",
        error
      );

      const navigatorOnline = navigator.onLine;
      setIsOnline(navigatorOnline);
      // Can't determine degradation status without backend, assume false
      setIsDegraded(false);
      updateStore(navigatorOnline, false);
    }
  }, [updateStore]);

  /**
   * Force a connectivity check immediately
   */
  const forceCheck = useCallback(async () => {
    try {
      const offlineStatus = await forceConnectivityCheck();

      setIsOnline(offlineStatus.isOnline);
      setIsDegraded(offlineStatus.isDegraded);
      updateStore(offlineStatus.isOnline, offlineStatus.isDegraded);
    } catch (error) {
      console.error("Force connectivity check failed:", error);

      // Fall back to navigator.onLine
      const navigatorOnline = navigator.onLine;
      setIsOnline(navigatorOnline);
      setIsDegraded(false);
      updateStore(navigatorOnline, false);
    }
  }, [updateStore]);

  /**
   * Handle online event
   */
  const handleOnline = useCallback(() => {
    console.debug("Online event detected");
    setIsOnline(true);
    setIsDegraded(false);
    updateStore(true, false);
  }, [updateStore]);

  /**
   * Handle offline event
   */
  const handleOffline = useCallback(() => {
    console.debug("Offline event detected");
    setIsOnline(false);
    setIsDegraded(false);
    updateStore(false, false);
  }, [updateStore]);

  /**
   * Initialize connectivity monitoring on mount
   */
  useEffect(() => {
    if (initializationRef.current) return;
    initializationRef.current = true;

    // Check initial state
    checkConnectivity();

    // Add event listeners for online/offline events
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    // Set up periodic connectivity check (every 30 seconds)
    // This helps detect degraded connections that navigator.onLine might miss
    pingIntervalRef.current = setInterval(() => {
      checkConnectivity();
    }, 30000);

    // Cleanup on unmount
    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);

      if (pingIntervalRef.current) {
        clearInterval(pingIntervalRef.current);
      }
    };
  }, [checkConnectivity, handleOnline, handleOffline]);

  return {
    isOnline,
    isDegraded,
    forceCheck,
  };
}
