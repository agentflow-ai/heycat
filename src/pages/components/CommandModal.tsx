import { useState, useEffect } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X, ChevronDown, ChevronUp } from "lucide-react";
import {
  Button,
  Input,
  Textarea,
  Label,
  FormField,
  Select,
  SelectItem,
} from "../../components/ui";
import type { CommandDto } from "../Commands";

type ActionType =
  | "open_app"
  | "type_text"
  | "system_control"
  | "workflow"
  | "custom";

const ACTION_TYPES: { value: ActionType; label: string }[] = [
  { value: "open_app", label: "Open Application" },
  { value: "type_text", label: "Type Text" },
  { value: "system_control", label: "System Control" },
  { value: "workflow", label: "Workflow" },
  { value: "custom", label: "Custom" },
];

const SYSTEM_CONTROLS = [
  { value: "volume_up", label: "Volume Up" },
  { value: "volume_down", label: "Volume Down" },
  { value: "volume_mute", label: "Mute" },
  { value: "play_pause", label: "Play/Pause" },
  { value: "next_track", label: "Next Track" },
  { value: "previous_track", label: "Previous Track" },
];

export interface CommandModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  command: CommandDto | null;
  existingTriggers: string[];
  onSave: (
    trigger: string,
    actionType: string,
    parameters: Record<string, string>
  ) => Promise<void>;
}

export function CommandModal({
  open,
  onOpenChange,
  command,
  existingTriggers,
  onSave,
}: CommandModalProps) {
  const [trigger, setTrigger] = useState("");
  const [actionType, setActionType] = useState<ActionType>("open_app");
  const [parameters, setParameters] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  // Reset form when modal opens/command changes
  useEffect(() => {
    if (open) {
      if (command) {
        setTrigger(command.trigger);
        setActionType(command.action_type as ActionType);
        setParameters(command.parameters);
        // Show advanced if any advanced params are set
        setShowAdvanced(
          Boolean(
            command.parameters.confirmation ||
              command.parameters.conditions ||
              command.parameters.custom_params
          )
        );
      } else {
        setTrigger("");
        setActionType("open_app");
        setParameters({});
        setShowAdvanced(false);
      }
      setErrors({});
    }
  }, [open, command]);

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!trigger.trim()) {
      newErrors.trigger = "Trigger phrase is required";
    } else if (existingTriggers.includes(trigger.trim().toLowerCase())) {
      newErrors.trigger = "This trigger phrase already exists";
    }

    switch (actionType) {
      case "open_app":
        if (!parameters.app?.trim()) {
          newErrors.app = "Application name is required";
        }
        break;
      case "type_text":
        if (!parameters.text?.trim()) {
          newErrors.text = "Text to type is required";
        }
        break;
      case "system_control":
        if (!parameters.control?.trim()) {
          newErrors.control = "Control type is required";
        }
        break;
      case "workflow":
        if (!parameters.workflow?.trim()) {
          newErrors.workflow = "Workflow name is required";
        }
        break;
      case "custom":
        if (!parameters.script?.trim()) {
          newErrors.script = "Script is required";
        }
        break;
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;

    try {
      setSaving(true);
      await onSave(trigger.trim(), actionType, parameters);
    } catch {
      // Error handled by parent via toast
    } finally {
      setSaving(false);
    }
  };

  const updateParameter = (key: string, value: string) => {
    setParameters((prev) => ({ ...prev, [key]: value }));
    if (errors[key]) {
      setErrors((prev) => {
        const next = { ...prev };
        delete next[key];
        return next;
      });
    }
  };

  const clearTriggerError = () => {
    if (errors.trigger) {
      setErrors((prev) => {
        const next = { ...prev };
        delete next.trigger;
        return next;
      });
    }
  };

  const renderParameterFields = () => {
    switch (actionType) {
      case "open_app":
        return (
          <FormField error={errors.app}>
            <Label htmlFor="param-app" required>
              Application
            </Label>
            <Input
              id="param-app"
              type="text"
              error={Boolean(errors.app)}
              value={parameters.app || ""}
              onChange={(e) => updateParameter("app", e.target.value)}
              placeholder="e.g., Safari, Calculator, Slack"
            />
          </FormField>
        );

      case "type_text":
        return (
          <>
            <FormField error={errors.text}>
              <Label htmlFor="param-text" required>
                Text to Type
              </Label>
              <Input
                id="param-text"
                type="text"
                error={Boolean(errors.text)}
                value={parameters.text || ""}
                onChange={(e) => updateParameter("text", e.target.value)}
                placeholder="Text that will be typed"
              />
            </FormField>
            <FormField>
              <Label htmlFor="param-delay">Delay (milliseconds)</Label>
              <Input
                id="param-delay"
                type="number"
                value={parameters.delay_ms || ""}
                onChange={(e) => updateParameter("delay_ms", e.target.value)}
                placeholder="0"
                min="0"
              />
            </FormField>
          </>
        );

      case "system_control":
        return (
          <FormField error={errors.control}>
            <Label htmlFor="param-control" required>
              Control Type
            </Label>
            <Select
              value={parameters.control || ""}
              onValueChange={(value) => updateParameter("control", value)}
              placeholder="Select control..."
            >
              {SYSTEM_CONTROLS.map((control) => (
                <SelectItem key={control.value} value={control.value}>
                  {control.label}
                </SelectItem>
              ))}
            </Select>
          </FormField>
        );

      case "workflow":
        return (
          <FormField error={errors.workflow}>
            <Label htmlFor="param-workflow" required>
              Workflow Name
            </Label>
            <Input
              id="param-workflow"
              type="text"
              error={Boolean(errors.workflow)}
              value={parameters.workflow || ""}
              onChange={(e) => updateParameter("workflow", e.target.value)}
              placeholder="Workflow name"
            />
          </FormField>
        );

      case "custom":
        return (
          <FormField error={errors.script}>
            <Label htmlFor="param-script" required>
              Script
            </Label>
            <Textarea
              id="param-script"
              error={Boolean(errors.script)}
              value={parameters.script || ""}
              onChange={(e) => updateParameter("script", e.target.value)}
              placeholder="Enter custom script..."
              rows={4}
            />
          </FormField>
        );
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 animate-in fade-in-0 z-50" />
        <Dialog.Content
          className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md bg-surface border border-border rounded-[var(--radius-lg)] shadow-xl animate-in fade-in-0 zoom-in-95 z-50"
          aria-describedby="command-modal-description"
        >
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <Dialog.Title className="text-lg font-semibold text-text-primary">
              {command ? "Edit Voice Command" : "Create Voice Command"}
            </Dialog.Title>
            <Dialog.Close asChild>
              <button
                type="button"
                className="p-1 rounded hover:bg-text-secondary/10 transition-colors text-text-secondary"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>
            </Dialog.Close>
          </div>

          <Dialog.Description id="command-modal-description" className="sr-only">
            {command
              ? `Edit the voice command "${command.trigger}"`
              : "Create a new voice command with a trigger phrase and action"}
          </Dialog.Description>

          {/* Form */}
          <form onSubmit={handleSubmit} className="p-4 space-y-4">
            {/* Trigger Phrase */}
            <FormField error={errors.trigger}>
              <Label htmlFor="trigger" required>
                Trigger Phrase
              </Label>
              <Input
                id="trigger"
                type="text"
                error={Boolean(errors.trigger)}
                value={trigger}
                onChange={(e) => {
                  setTrigger(e.target.value);
                  clearTriggerError();
                }}
                placeholder='e.g., "open browser"'
              />
            </FormField>

            {/* Action Type */}
            <FormField>
              <Label htmlFor="action-type">Action Type</Label>
              <Select
                value={actionType}
                onValueChange={(value) => {
                  setActionType(value as ActionType);
                  setParameters({});
                }}
              >
                {ACTION_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </Select>
            </FormField>

            {/* Dynamic Parameter Fields */}
            {renderParameterFields()}

            {/* Progressive Disclosure: Advanced Options */}
            <div className="border-t border-border pt-4">
              <button
                type="button"
                className="flex items-center gap-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
                onClick={() => setShowAdvanced(!showAdvanced)}
                aria-expanded={showAdvanced}
              >
                {showAdvanced ? (
                  <ChevronUp className="h-4 w-4" />
                ) : (
                  <ChevronDown className="h-4 w-4" />
                )}
                Advanced Options
              </button>

              {showAdvanced && (
                <div className="mt-4 space-y-4">
                  {/* Custom Parameters */}
                  <FormField>
                    <Label htmlFor="custom-params">Custom Parameters</Label>
                    <Input
                      id="custom-params"
                      type="text"
                      value={parameters.custom_params || ""}
                      onChange={(e) =>
                        updateParameter("custom_params", e.target.value)
                      }
                      placeholder="key=value, key2=value2"
                    />
                  </FormField>

                  {/* Conditions */}
                  <FormField>
                    <Label htmlFor="conditions">Conditions/Context</Label>
                    <Input
                      id="conditions"
                      type="text"
                      value={parameters.conditions || ""}
                      onChange={(e) =>
                        updateParameter("conditions", e.target.value)
                      }
                      placeholder="e.g., app=Safari, time=morning"
                    />
                  </FormField>

                  {/* Confirmation Toggle */}
                  <div className="flex items-center justify-between">
                    <div>
                      <Label htmlFor="confirmation">Require Confirmation</Label>
                      <p className="text-xs text-text-secondary">
                        Ask before executing this command
                      </p>
                    </div>
                    <input
                      id="confirmation"
                      type="checkbox"
                      checked={parameters.confirmation === "true"}
                      onChange={(e) =>
                        updateParameter(
                          "confirmation",
                          e.target.checked ? "true" : ""
                        )
                      }
                      className="h-4 w-4 rounded border-border text-heycat-orange focus:ring-heycat-teal"
                    />
                  </div>
                </div>
              )}
            </div>

            {/* Footer Buttons */}
            <div className="flex justify-end gap-3 pt-4 border-t border-border">
              <Button
                type="button"
                variant="secondary"
                onClick={() => onOpenChange(false)}
                disabled={saving}
              >
                Cancel
              </Button>
              <Button type="submit" loading={saving}>
                {command ? "Save Changes" : "Save Command"}
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
