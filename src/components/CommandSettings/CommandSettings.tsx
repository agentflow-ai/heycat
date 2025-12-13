import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { CommandList } from "./CommandList";
import { CommandEditor } from "./CommandEditor";
import "./CommandSettings.css";

export interface CommandDto {
  id: string;
  trigger: string;
  action_type: string;
  parameters: Record<string, string>;
  enabled: boolean;
}

export interface CommandSettingsProps {
  className?: string;
}

export function CommandSettings({ className = "" }: CommandSettingsProps) {
  const [commands, setCommands] = useState<CommandDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingCommand, setEditingCommand] = useState<CommandDto | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  const loadCommands = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<CommandDto[]>("get_commands");
      setCommands(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadCommands();
  }, [loadCommands]);

  const handleAddCommand = async (
    trigger: string,
    actionType: string,
    parameters: Record<string, string>
  ) => {
    try {
      const newCommand = await invoke<CommandDto>("add_command", {
        input: {
          trigger,
          action_type: actionType,
          parameters,
          enabled: true,
        },
      });
      setCommands((prev) => [...prev, newCommand]);
      setIsAdding(false);
    } catch (e) {
      throw e;
    }
  };

  const handleEditCommand = async (
    trigger: string,
    actionType: string,
    parameters: Record<string, string>
  ) => {
    if (!editingCommand) return;

    try {
      // Remove old command and add new one with same trigger updates
      await invoke("remove_command", { id: editingCommand.id });
      const updatedCommand = await invoke<CommandDto>("add_command", {
        input: {
          trigger,
          action_type: actionType,
          parameters,
          enabled: editingCommand.enabled,
        },
      });
      setCommands((prev) =>
        prev.map((c) => (c.id === editingCommand.id ? updatedCommand : c))
      );
      setEditingCommand(null);
    } catch (e) {
      throw e;
    }
  };

  const handleDeleteCommand = async (id: string) => {
    try {
      await invoke("remove_command", { id });
      setCommands((prev) => prev.filter((c) => c.id !== id));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleToggleEnabled = async (command: CommandDto) => {
    try {
      // Remove and re-add with toggled enabled state
      await invoke("remove_command", { id: command.id });
      const updatedCommand = await invoke<CommandDto>("add_command", {
        input: {
          trigger: command.trigger,
          action_type: command.action_type,
          parameters: command.parameters,
          enabled: !command.enabled,
        },
      });
      setCommands((prev) =>
        prev.map((c) => (c.id === command.id ? updatedCommand : c))
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  if (loading) {
    return (
      <div className={`command-settings ${className}`.trim()}>
        <div className="command-settings__loading" role="status">
          Loading commands...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`command-settings ${className}`.trim()}>
        <div className="command-settings__error" role="alert">
          {error}
        </div>
      </div>
    );
  }

  const showEditor = isAdding || editingCommand !== null;

  return (
    <div className={`command-settings ${className}`.trim()}>
      <div className="command-settings__header">
        <h2 className="command-settings__title">Voice Commands</h2>
        {!showEditor && (
          <button
            className="command-settings__add-button"
            onClick={() => setIsAdding(true)}
            type="button"
          >
            Add Command
          </button>
        )}
      </div>

      {showEditor && (
        <CommandEditor
          command={editingCommand}
          existingTriggers={commands
            .filter((c) => c.id !== editingCommand?.id)
            .map((c) => c.trigger)}
          onSave={editingCommand ? handleEditCommand : handleAddCommand}
          onCancel={() => {
            setIsAdding(false);
            setEditingCommand(null);
          }}
        />
      )}

      <CommandList
        commands={commands}
        onEdit={setEditingCommand}
        onDelete={handleDeleteCommand}
        onToggleEnabled={handleToggleEnabled}
      />
    </div>
  );
}
