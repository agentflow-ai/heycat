import { useState, useEffect, useCallback } from "react";
import {
  createHashRouter,
  Navigate,
  Outlet,
  useNavigate,
  useLocation,
  useOutletContext,
} from "react-router-dom";
import { Dashboard, Commands, Recordings, Settings, Dictionary, WindowContexts } from "./pages";
import { AppShell } from "./components/layout/AppShell";
import { useAppStatus } from "./hooks/useAppStatus";
import { useRecording } from "./hooks/useRecording";
import { useSettings } from "./hooks/useSettings";
import { useCatOverlay } from "./hooks/useCatOverlay";

/**
 * Maps URL pathname to nav item ID.
 * Handles the root path "/" mapping to "dashboard".
 */
function pathToNavItem(pathname: string): string {
  if (pathname === "/" || pathname === "") return "dashboard";
  // Remove leading slash
  return pathname.slice(1);
}

/**
 * Maps nav item ID to URL pathname.
 */
function navItemToPath(navItem: string): string {
  if (navItem === "dashboard") return "/";
  return `/${navItem}`;
}

/**
 * Root layout component that wraps all pages with AppShell.
 * Integrates React Router navigation with the sidebar navigation.
 */
function RootLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const { status: appStatus, isRecording } = useAppStatus();
  const [recordingDuration, setRecordingDuration] = useState(0);
  useCatOverlay();

  // Get settings for device name
  const { settings } = useSettings();

  // Get recording actions
  const { startRecording, stopRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });

  // Derive active nav item from current URL
  const activeNavItem = pathToNavItem(location.pathname);

  // Handle navigation from AppShell/pages
  const handleNavigate = useCallback(
    (navItem: string) => {
      navigate(navItemToPath(navItem));
    },
    [navigate]
  );

  // Track recording duration
  useEffect(() => {
    if (!isRecording) {
      setRecordingDuration(0);
      return;
    }
    setRecordingDuration(0);
    const interval = setInterval(() => {
      setRecordingDuration((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [isRecording]);

  return (
    <AppShell
      activeNavItem={activeNavItem}
      onNavigate={handleNavigate}
      status={appStatus}
      recordingDuration={isRecording ? recordingDuration : undefined}
      footerStateDescription="Ready for your command."
      isRecording={isRecording}
      onStartRecording={startRecording}
      onStopRecording={stopRecording}
    >
      <Outlet context={{ onNavigate: handleNavigate }} />
    </AppShell>
  );
}

/**
 * Application router using hash-based routing for Tauri compatibility.
 *
 * Routes:
 * - `/` (index) - Dashboard
 * - `/commands` - Commands page
 * - `/recordings` - Recordings page
 * - `/settings` - Settings page
 * - `*` - Redirects to Dashboard (404 fallback)
 *
 * Note: Using createHashRouter instead of createBrowserRouter because
 * Tauri's file:// protocol doesn't support the browser history API well.
 */
export const router = createHashRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      { index: true, element: <Dashboard /> },
      { path: "commands", element: <Commands /> },
      { path: "recordings", element: <Recordings /> },
      { path: "settings", element: <Settings /> },
      { path: "dictionary", element: <Dictionary /> },
      { path: "contexts", element: <WindowContexts /> },
      // Catch-all route redirects to dashboard
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);

/**
 * Type for the outlet context provided by RootLayout.
 * Pages can use this to navigate via the legacy onNavigate pattern.
 */
export interface RouteOutletContext {
  onNavigate: (page: string) => void;
}

/**
 * Hook to access the route outlet context from page components.
 * Provides the onNavigate function for backward compatibility with pages.
 * Returns null if used outside of a router context (e.g., in tests).
 *
 * @example
 * ```tsx
 * function MyPage() {
 *   const context = useRouteContext();
 *   return <button onClick={() => context?.onNavigate('settings')}>Settings</button>;
 * }
 * ```
 */
export function useRouteContext(): RouteOutletContext | null {
  try {
    return useOutletContext<RouteOutletContext>();
  } catch {
    // Return null when used outside of router context (e.g., in tests)
    return null;
  }
}
