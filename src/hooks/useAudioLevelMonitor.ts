import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface UseAudioLevelMonitorOptions {
  /** Device name to monitor (null = system default) */
  deviceName: string | null;
  /** Whether monitoring is enabled (default: true) */
  enabled?: boolean;
}

export interface UseAudioLevelMonitorResult {
  /** Current audio level (0-100) */
  level: number;
  /** Whether the monitor is currently active */
  isMonitoring: boolean;
}

/**
 * Hook for real-time audio level monitoring.
 *
 * Starts capturing audio from the specified device and provides level updates
 * at ~20fps for visual feedback in the device selector.
 *
 * @param options Configuration options
 * @returns Current level and monitoring state
 */
export function useAudioLevelMonitor({
  deviceName,
  enabled = true,
}: UseAudioLevelMonitorOptions): UseAudioLevelMonitorResult {
  const [level, setLevel] = useState(0);
  const [isMonitoring, setIsMonitoring] = useState(false);
  // Use ref to store latest level for throttled updates
  const levelRef = useRef(0);

  useEffect(() => {
    if (!enabled) {
      setLevel(0);
      setIsMonitoring(false);
      return;
    }

    let cancelled = false;
    let unlistenFn: (() => void) | null = null;
    let intervalId: ReturnType<typeof setInterval> | null = null;

    const startMonitor = async () => {
      try {
        // Start backend monitoring
        await invoke("start_audio_monitor", {
          deviceName: deviceName ?? undefined,
        });
        if (!cancelled) {
          setIsMonitoring(true);
        }
      } catch (e) {
        console.error("Failed to start audio monitor:", e);
        if (!cancelled) {
          setIsMonitoring(false);
        }
      }
    };

    const stopMonitor = async () => {
      try {
        await invoke("stop_audio_monitor");
      } catch (e) {
        console.error("Failed to stop audio monitor:", e);
      }
      setIsMonitoring(false);
      setLevel(0);
    };

    const setupListener = async () => {
      // Listen for level events from backend
      unlistenFn = await listen<number>("audio-level", (event) => {
        levelRef.current = event.payload;
      });

      // Update state at controlled rate (20fps) to avoid excessive renders
      // Backend may emit faster than needed, so we throttle here
      intervalId = setInterval(() => {
        setLevel(levelRef.current);
      }, 50);
    };

    // Start monitoring and listening
    setupListener();
    startMonitor();

    return () => {
      cancelled = true;
      if (intervalId) {
        clearInterval(intervalId);
      }
      if (unlistenFn) {
        unlistenFn();
      }
      stopMonitor();
    };
  }, [deviceName, enabled]);

  return { level, isMonitoring };
}
