import { useState } from "react";
import type { CommandDto } from "./CommandSettings";

export interface CommandListProps {
  commands: CommandDto[];
  onEdit: (command: CommandDto) => void;
  onDelete: (id: string) => void;
  onToggleEnabled: (command: CommandDto) => void;
}

const ACTION_TYPE_LABELS: Record<string, string> = {
  open_app: "Open App",
  type_text: "Type Text",
  system_control: "System Control",
  workflow: "Workflow",
  custom: "Custom",
};

export function CommandList({
  commands,
  onEdit,
  onDelete,
  onToggleEnabled,
}: CommandListProps) {
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  const handleDeleteClick = (id: string) => {
    setDeleteConfirmId(id);
  };

  const handleConfirmDelete = (id: string) => {
    onDelete(id);
    setDeleteConfirmId(null);
  };

  const handleCancelDelete = () => {
    setDeleteConfirmId(null);
  };

  if (commands.length === 0) {
    return (
      <div className="command-list__empty" role="status" aria-live="polite">
        <span className="command-list__empty-icon" aria-hidden="true">
          No commands configured
        </span>
        <p className="command-list__empty-text">
          Add your first voice command to get started
        </p>
      </div>
    );
  }

  return (
    <ul className="command-list" role="list">
      {commands.map((command) => (
        <li key={command.id} className="command-list__item">
          <div className="command-list__info">
            <span className="command-list__trigger">{command.trigger}</span>
            <span className="command-list__action-type">
              {ACTION_TYPE_LABELS[command.action_type] || command.action_type}
            </span>
          </div>

          <div className="command-list__actions">
            <label className="command-list__toggle">
              <input
                type="checkbox"
                checked={command.enabled}
                onChange={() => onToggleEnabled(command)}
                aria-label={`${command.enabled ? "Disable" : "Enable"} ${command.trigger}`}
              />
              <span className="command-list__toggle-slider" />
            </label>

            <button
              className="command-list__edit-button"
              onClick={() => onEdit(command)}
              type="button"
              aria-label={`Edit ${command.trigger}`}
            >
              Edit
            </button>

            {deleteConfirmId === command.id ? (
              <div className="command-list__delete-confirm">
                <button
                  className="command-list__confirm-button"
                  onClick={() => handleConfirmDelete(command.id)}
                  type="button"
                  aria-label="Confirm delete"
                >
                  Confirm
                </button>
                <button
                  className="command-list__cancel-button"
                  onClick={handleCancelDelete}
                  type="button"
                  aria-label="Cancel delete"
                >
                  Cancel
                </button>
              </div>
            ) : (
              <button
                className="command-list__delete-button"
                onClick={() => handleDeleteClick(command.id)}
                type="button"
                aria-label={`Delete ${command.trigger}`}
              >
                Delete
              </button>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}
