// Window context types for context-sensitive commands

export interface ActiveWindowInfo {
  appName: string;
  bundleId?: string;
  windowTitle?: string;
  pid: number;
}

export interface WindowMatcher {
  appName: string;
  titlePattern?: string;
  bundleId?: string;
}

export type OverrideMode = "merge" | "replace";

export interface WindowContext {
  id: string;
  name: string;
  matcher: WindowMatcher;
  commandMode: OverrideMode;
  dictionaryMode: OverrideMode;
  commandIds: string[];
  dictionaryEntryIds: string[];
  enabled: boolean;
  priority: number;
}

export interface ActiveWindowChangedPayload {
  appName: string;
  bundleId?: string;
  windowTitle?: string;
  matchedContextId?: string;
  matchedContextName?: string;
}

export interface RunningApplication {
  /** The localized name of the application */
  name: string;
  /** The bundle identifier (e.g., "com.apple.Safari") */
  bundleId?: string;
  /** Whether this is the currently active (frontmost) application */
  isActive: boolean;
}
