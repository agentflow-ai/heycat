import { useState, useEffect } from "react";
import type { CommandDto } from "./CommandSettings";

export interface CommandEditorProps {
  command: CommandDto | null;
  existingTriggers: string[];
  onSave: (
    trigger: string,
    actionType: string,
    parameters: Record<string, string>
  ) => Promise<void>;
  onCancel: () => void;
}

type ActionType =
  | "open_app"
  | "type_text"
  | "system_control"
  | "workflow"
  | "custom";

const ACTION_TYPES: { value: ActionType; label: string }[] = [
  { value: "open_app", label: "Open App" },
  { value: "type_text", label: "Type Text" },
  { value: "system_control", label: "System Control" },
  { value: "workflow", label: "Workflow" },
  { value: "custom", label: "Custom" },
];

const SYSTEM_CONTROLS = [
  "volume_up",
  "volume_down",
  "volume_mute",
  "play_pause",
  "next_track",
  "previous_track",
];

export function CommandEditor({
  command,
  existingTriggers,
  onSave,
  onCancel,
}: CommandEditorProps) {
  const [trigger, setTrigger] = useState(command?.trigger || "");
  const [actionType, setActionType] = useState<ActionType>(
    (command?.action_type as ActionType) || "open_app"
  );
  const [parameters, setParameters] = useState<Record<string, string>>(
    command?.parameters || {}
  );
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (command) {
      setTrigger(command.trigger);
      setActionType(command.action_type as ActionType);
      setParameters(command.parameters);
    } else {
      setTrigger("");
      setActionType("open_app");
      setParameters({});
    }
    setErrors({});
  }, [command]);

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!trigger.trim()) {
      newErrors.trigger = "Trigger is required";
    } else if (existingTriggers.includes(trigger.trim().toLowerCase())) {
      newErrors.trigger = "Trigger already exists";
    }

    switch (actionType) {
      case "open_app":
        if (!parameters.app?.trim()) {
          newErrors.app = "App name is required";
        }
        break;
      case "type_text":
        if (!parameters.text?.trim()) {
          newErrors.text = "Text is required";
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
    } catch (err) {
      setErrors({ submit: err instanceof Error ? err.message : String(err) });
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

  const renderParameterFields = () => {
    switch (actionType) {
      case "open_app":
        return (
          <div className="command-editor__field">
            <label htmlFor="param-app" className="command-editor__label">
              App Name
            </label>
            <input
              id="param-app"
              type="text"
              className={`command-editor__input ${errors.app ? "command-editor__input--error" : ""}`}
              value={parameters.app || ""}
              onChange={(e) => updateParameter("app", e.target.value)}
              placeholder="e.g., Safari, Calculator"
            />
            {errors.app && (
              <span className="command-editor__error" role="alert">
                {errors.app}
              </span>
            )}
          </div>
        );

      case "type_text":
        return (
          <>
            <div className="command-editor__field">
              <label htmlFor="param-text" className="command-editor__label">
                Text to Type
              </label>
              <input
                id="param-text"
                type="text"
                className={`command-editor__input ${errors.text ? "command-editor__input--error" : ""}`}
                value={parameters.text || ""}
                onChange={(e) => updateParameter("text", e.target.value)}
                placeholder="Text to insert"
              />
              {errors.text && (
                <span className="command-editor__error" role="alert">
                  {errors.text}
                </span>
              )}
            </div>
            <div className="command-editor__field">
              <label htmlFor="param-delay" className="command-editor__label">
                Delay (ms)
              </label>
              <input
                id="param-delay"
                type="number"
                className="command-editor__input"
                value={parameters.delay_ms || ""}
                onChange={(e) => updateParameter("delay_ms", e.target.value)}
                placeholder="0"
                min="0"
              />
            </div>
          </>
        );

      case "system_control":
        return (
          <div className="command-editor__field">
            <label htmlFor="param-control" className="command-editor__label">
              Control Type
            </label>
            <select
              id="param-control"
              className={`command-editor__select ${errors.control ? "command-editor__input--error" : ""}`}
              value={parameters.control || ""}
              onChange={(e) => updateParameter("control", e.target.value)}
            >
              <option value="">Select control...</option>
              {SYSTEM_CONTROLS.map((control) => (
                <option key={control} value={control}>
                  {control.replace(/_/g, " ")}
                </option>
              ))}
            </select>
            {errors.control && (
              <span className="command-editor__error" role="alert">
                {errors.control}
              </span>
            )}
          </div>
        );

      case "workflow":
        return (
          <div className="command-editor__field">
            <label htmlFor="param-workflow" className="command-editor__label">
              Workflow Name
            </label>
            <input
              id="param-workflow"
              type="text"
              className={`command-editor__input ${errors.workflow ? "command-editor__input--error" : ""}`}
              value={parameters.workflow || ""}
              onChange={(e) => updateParameter("workflow", e.target.value)}
              placeholder="Workflow name"
            />
            {errors.workflow && (
              <span className="command-editor__error" role="alert">
                {errors.workflow}
              </span>
            )}
          </div>
        );

      case "custom":
        return (
          <div className="command-editor__field">
            <label htmlFor="param-script" className="command-editor__label">
              Script
            </label>
            <textarea
              id="param-script"
              className={`command-editor__textarea ${errors.script ? "command-editor__input--error" : ""}`}
              value={parameters.script || ""}
              onChange={(e) => updateParameter("script", e.target.value)}
              placeholder="Enter custom script..."
              rows={4}
            />
            {errors.script && (
              <span className="command-editor__error" role="alert">
                {errors.script}
              </span>
            )}
          </div>
        );
    }
  };

  return (
    <form className="command-editor" onSubmit={handleSubmit}>
      <h3 className="command-editor__heading">
        {command ? "Edit Command" : "Add Command"}
      </h3>

      {errors.submit && (
        <div className="command-editor__submit-error" role="alert">
          {errors.submit}
        </div>
      )}

      <div className="command-editor__field">
        <label htmlFor="trigger" className="command-editor__label">
          Trigger Phrase
        </label>
        <input
          id="trigger"
          type="text"
          className={`command-editor__input ${errors.trigger ? "command-editor__input--error" : ""}`}
          value={trigger}
          onChange={(e) => {
            setTrigger(e.target.value);
            if (errors.trigger) {
              setErrors((prev) => {
                const next = { ...prev };
                delete next.trigger;
                return next;
              });
            }
          }}
          placeholder='e.g., "open browser"'
        />
        {errors.trigger && (
          <span className="command-editor__error" role="alert">
            {errors.trigger}
          </span>
        )}
      </div>

      <div className="command-editor__field">
        <label htmlFor="action-type" className="command-editor__label">
          Action Type
        </label>
        <select
          id="action-type"
          className="command-editor__select"
          value={actionType}
          onChange={(e) => {
            setActionType(e.target.value as ActionType);
            setParameters({});
          }}
        >
          {ACTION_TYPES.map((type) => (
            <option key={type.value} value={type.value}>
              {type.label}
            </option>
          ))}
        </select>
      </div>

      {renderParameterFields()}

      <div className="command-editor__buttons">
        <button
          type="button"
          className="command-editor__cancel-button"
          onClick={onCancel}
          disabled={saving}
        >
          Cancel
        </button>
        <button
          type="submit"
          className="command-editor__save-button"
          disabled={saving}
        >
          {saving ? "Saving..." : command ? "Save Changes" : "Add Command"}
        </button>
      </div>
    </form>
  );
}
