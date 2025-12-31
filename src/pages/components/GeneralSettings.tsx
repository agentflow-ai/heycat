import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import * as RadioGroupPrimitive from "@radix-ui/react-radio-group";
import { queryKeys } from "../../lib/queryKeys";
import { Card, CardContent, LabeledToggle, Button } from "../../components/ui";
import { useSettings, RecordingMode } from "../../hooks/useSettings";
import { useRecordingState } from "../../hooks/useRecording";
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
  const { settings, updateDistinguishLeftRight, updateRecordingMode } = useSettings();
  const { isRecording, isProcessing } = useRecordingState();
  const { toast } = useToast();
  const queryClient = useQueryClient();

  // Disable recording mode changes while recording is active
  const isRecordingActive = isRecording || isProcessing;

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

  const handleRecordingModeChange = async (mode: RecordingMode) => {
    try {
      // Update backend via Tauri command
      await invoke("set_recording_mode", { mode });
      // Update frontend settings
      await updateRecordingMode(mode);
      toast({
        type: "success",
        title: "Setting saved",
        description: mode === "toggle"
          ? "Recording mode set to Toggle (press to start/stop)."
          : "Recording mode set to Push-to-Talk (hold to record).",
      });
    } catch (error) {
      toast({
        type: "error",
        title: "Failed to update recording mode",
        description: String(error),
      });
    }
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
            {/* Recording Mode */}
            <div className="py-2">
              <div className="flex flex-col gap-3">
                <div>
                  <span className="text-sm font-medium text-text-primary">
                    Recording Mode
                  </span>
                  <p className="text-xs text-text-secondary mt-0.5">
                    Choose how the recording shortcut behaves
                  </p>
                </div>
                <RadioGroupPrimitive.Root
                  value={settings.shortcuts.recordingMode}
                  onValueChange={(value) => handleRecordingModeChange(value as RecordingMode)}
                  disabled={isRecordingActive}
                  className="flex flex-col gap-2"
                >
                  <label className={`flex items-center gap-3 cursor-pointer ${isRecordingActive ? 'opacity-50 cursor-not-allowed' : ''}`}>
                    <RadioGroupPrimitive.Item
                      value="toggle"
                      className="h-4 w-4 rounded-full border border-neutral-400 bg-white focus:outline-none focus:ring-2 focus:ring-heycat-teal focus:ring-offset-1 data-[state=checked]:border-heycat-orange data-[state=checked]:bg-heycat-orange disabled:cursor-not-allowed"
                    >
                      <RadioGroupPrimitive.Indicator className="flex items-center justify-center">
                        <div className="h-1.5 w-1.5 rounded-full bg-white" />
                      </RadioGroupPrimitive.Indicator>
                    </RadioGroupPrimitive.Item>
                    <div className="flex flex-col">
                      <span className="text-sm text-text-primary">Toggle</span>
                      <span className="text-xs text-text-secondary">Press to start, press again to stop</span>
                    </div>
                  </label>
                  <label className={`flex items-center gap-3 cursor-pointer ${isRecordingActive ? 'opacity-50 cursor-not-allowed' : ''}`}>
                    <RadioGroupPrimitive.Item
                      value="push-to-talk"
                      className="h-4 w-4 rounded-full border border-neutral-400 bg-white focus:outline-none focus:ring-2 focus:ring-heycat-teal focus:ring-offset-1 data-[state=checked]:border-heycat-orange data-[state=checked]:bg-heycat-orange disabled:cursor-not-allowed"
                    >
                      <RadioGroupPrimitive.Indicator className="flex items-center justify-center">
                        <div className="h-1.5 w-1.5 rounded-full bg-white" />
                      </RadioGroupPrimitive.Indicator>
                    </RadioGroupPrimitive.Item>
                    <div className="flex flex-col">
                      <span className="text-sm text-text-primary">Push-to-Talk</span>
                      <span className="text-xs text-text-secondary">Hold to record, release to stop</span>
                    </div>
                  </label>
                </RadioGroupPrimitive.Root>
                {isRecordingActive && (
                  <p className="text-xs text-amber-600">
                    Recording mode cannot be changed while recording
                  </p>
                )}
              </div>
            </div>

            {/* Recording Shortcut */}
            <div className="flex items-center justify-between py-2 border-t border-border">
              <div>
                <span className="text-sm font-medium text-text-primary">
                  Recording Shortcut
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
