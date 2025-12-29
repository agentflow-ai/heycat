import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../../lib/queryKeys";
import { Card, CardContent, LabeledToggle, Button } from "../../components/ui";
import { useSettings } from "../../hooks/useSettings";
import { useToast } from "../../components/overlays";
import { ShortcutEditor } from "./ShortcutEditor";

export interface GeneralSettingsProps {
  className?: string;
}

// Convert backend shortcut format to display format
function backendToDisplay(shortcut: string): string {
  return shortcut
    .replace(/Function/gi, "fn")
    .replace(/CmdOrControl/gi, "⌘")
    .replace(/Ctrl/gi, "⌃")
    .replace(/Alt/gi, "⌥")
    .replace(/Shift/gi, "⇧")
    .replace(/\+/g, "");
}

export function GeneralSettings({ className = "" }: GeneralSettingsProps) {
  const { settings, updateDistinguishLeftRight } = useSettings();
  const { toast } = useToast();
  const queryClient = useQueryClient();

  // Local state for settings that don't have hooks yet
  const [launchAtLogin, setLaunchAtLogin] = useState(false);
  const [notifications, setNotifications] = useState(true);

  // Fetch recording shortcut via React Query
  const { data: backendShortcut } = useQuery({
    queryKey: queryKeys.tauri.recordingShortcut,
    queryFn: () => invoke<string>("get_recording_shortcut"),
  });

  const currentShortcut = backendShortcut ? backendToDisplay(backendShortcut) : "⌘⇧R";

  // Shortcut editor modal state
  const [isShortcutEditorOpen, setIsShortcutEditorOpen] = useState(false);

  const handleLaunchAtLoginChange = async (checked: boolean) => {
    setLaunchAtLogin(checked);
    toast({
      type: "success",
      title: "Setting saved",
      description: `Launch at login ${checked ? "enabled" : "disabled"}.`,
    });
  };

  const handleNotificationsChange = async (checked: boolean) => {
    setNotifications(checked);
    toast({
      type: "success",
      title: "Setting saved",
      description: `Notifications ${checked ? "enabled" : "disabled"}.`,
    });
  };

  const handleDistinguishLeftRightChange = async (checked: boolean) => {
    await updateDistinguishLeftRight(checked);
    toast({
      type: "success",
      title: "Setting saved",
      description: `Left/Right modifier distinction ${checked ? "enabled" : "disabled"}.`,
    });
  };

  return (
    <div className={`space-y-6 ${className}`.trim()}>
      {/* General Settings Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          General
        </h2>
        <Card>
          <CardContent className="space-y-4 divide-y divide-border">
            <div className="pt-0">
              <LabeledToggle
                label="Launch at Login"
                description="Start HeyCat when you log in to your Mac"
                checked={launchAtLogin}
                onCheckedChange={handleLaunchAtLoginChange}
              />
            </div>
            <div className="pt-4">
              <LabeledToggle
                label="Notifications"
                description="Show notifications for transcription results"
                checked={notifications}
                onCheckedChange={handleNotificationsChange}
              />
            </div>
          </CardContent>
        </Card>
      </section>

      {/* Keyboard Shortcuts Section */}
      <section>
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider mb-4">
          Keyboard Shortcuts
        </h2>
        <Card>
          <CardContent className="space-y-3">
            {/* Toggle Recording */}
            <div className="flex items-center justify-between py-2">
              <div>
                <span className="text-sm font-medium text-text-primary">
                  Toggle Recording
                </span>
              </div>
              <div className="flex items-center gap-2">
                <kbd className="px-2 py-1 text-xs font-mono bg-surface border border-border rounded">
                  {currentShortcut}
                </kbd>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setIsShortcutEditorOpen(true)}
                >
                  Change
                </Button>
              </div>
            </div>

            {/* Cancel Recording */}
            <div className="flex items-center justify-between py-2 border-t border-border">
              <div>
                <span className="text-sm font-medium text-text-primary">
                  Cancel Recording
                </span>
              </div>
              <kbd className="px-2 py-1 text-xs font-mono bg-surface border border-border rounded">
                Esc Esc
              </kbd>
            </div>

            {/* Open Command Palette */}
            <div className="flex items-center justify-between py-2 border-t border-border">
              <div>
                <span className="text-sm font-medium text-text-primary">
                  Open Command Palette
                </span>
              </div>
              <kbd className="px-2 py-1 text-xs font-mono bg-surface border border-border rounded">
                ⌘K
              </kbd>
            </div>

            {/* Distinguish Left/Right Modifiers */}
            <div className="pt-4 border-t border-border">
              <LabeledToggle
                label="Distinguish Left/Right Modifiers"
                description="When enabled, Left-Command and Right-Command are treated as different keys"
                checked={settings.shortcuts.distinguishLeftRight}
                onCheckedChange={handleDistinguishLeftRightChange}
              />
            </div>
          </CardContent>
        </Card>
      </section>

      {/* Shortcut Editor Modal */}
      <ShortcutEditor
        open={isShortcutEditorOpen}
        onOpenChange={setIsShortcutEditorOpen}
        shortcutName="Toggle Recording"
        currentShortcut={currentShortcut}
        onSave={async (displayShortcut, newBackendShortcut) => {
          try {
            await invoke("update_recording_shortcut", { newShortcut: newBackendShortcut });
            // Invalidate to refetch the updated shortcut
            await queryClient.invalidateQueries({ queryKey: queryKeys.tauri.recordingShortcut });
            toast({
              type: "success",
              title: "Shortcut updated",
              description: `Toggle Recording is now ${displayShortcut}.`,
            });
          } catch (error) {
            console.error("Failed to update shortcut:", error);
            toast({
              type: "error",
              title: "Failed to update shortcut",
              description: String(error),
            });
          }
          setIsShortcutEditorOpen(false);
        }}
      />
    </div>
  );
}
