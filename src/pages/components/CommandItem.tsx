import { Pencil, X, Layers } from "lucide-react";
import { Card, CardContent, Button, Toggle } from "../../components/ui";
import type { CommandDto } from "../Commands";
import type { WindowContext } from "../../types/windowContext";

const ACTION_TYPE_LABELS: Record<string, string> = {
  open_app: "Open App",
  type_text: "Type Text",
  system_control: "System Control",
  custom: "Custom",
};

const ACTION_TYPE_COLORS: Record<string, string> = {
  open_app: "bg-heycat-teal/10 text-heycat-teal",
  type_text: "bg-heycat-purple/10 text-heycat-purple",
  system_control: "bg-heycat-orange/10 text-heycat-orange",
  custom: "bg-text-secondary/10 text-text-secondary",
};

interface ContextBadgesProps {
  contexts: WindowContext[];
}

function ContextBadges({ contexts }: ContextBadgesProps) {
  if (contexts.length === 0) {
    return (
      <span
        className="text-xs px-2 py-0.5 rounded bg-neutral-100 text-neutral-600 dark:bg-neutral-800 dark:text-neutral-400"
        data-testid="context-badge-global"
      >
        Global
      </span>
    );
  }

  if (contexts.length === 1) {
    return (
      <span
        className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
        data-testid="context-badge"
      >
        <Layers className="h-3 w-3" />
        {contexts[0].name}
      </span>
    );
  }

  // For 2+ contexts, show count
  return (
    <span
      className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300"
      data-testid="context-badge"
      title={contexts.map((c) => c.name).join(", ")}
    >
      <Layers className="h-3 w-3" />
      {contexts.length} contexts
    </span>
  );
}

export interface CommandItemProps {
  command: CommandDto;
  /** Window contexts this command is assigned to */
  assignedContexts: WindowContext[];
  onEdit: (command: CommandDto) => void;
  onDelete: (id: string) => void;
  onToggleEnabled: (command: CommandDto) => void;
  isDeleting?: boolean;
  onConfirmDelete?: (id: string) => void;
  onCancelDelete?: () => void;
}

function getActionDescription(command: CommandDto): string {
  switch (command.action_type) {
    case "open_app":
      return command.parameters.app
        ? `Opens ${command.parameters.app}`
        : "Opens an application";
    case "type_text":
      return command.parameters.text
        ? `Types: ${command.parameters.text}`
        : "Types text";
    case "system_control":
      return command.parameters.control
        ? `${command.parameters.control.replace(/_/g, " ")}`
        : "System control";
    case "custom":
      return "Custom script";
    default:
      return "";
  }
}

export function CommandItem({
  command,
  assignedContexts,
  onEdit,
  onDelete,
  onToggleEnabled,
  isDeleting = false,
  onConfirmDelete,
  onCancelDelete,
}: CommandItemProps) {
  const actionLabel =
    ACTION_TYPE_LABELS[command.action_type] || command.action_type;
  const badgeColor =
    ACTION_TYPE_COLORS[command.action_type] ||
    "bg-text-secondary/10 text-text-secondary";
  const description = getActionDescription(command);

  return (
    <Card
      className={`transition-opacity ${!command.enabled ? "opacity-60" : ""}`}
      role="listitem"
    >
      <CardContent className="flex items-center gap-4 py-3">
        {/* Toggle */}
        <Toggle
          checked={command.enabled}
          onCheckedChange={() => onToggleEnabled(command)}
          aria-label={`${command.enabled ? "Disable" : "Enable"} ${command.trigger}`}
        />

        {/* Command Info */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3">
            {/* Trigger phrase in quotes */}
            <span className="text-sm font-medium text-text-primary">
              "{command.trigger}"
            </span>
            {/* Action type badge */}
            <span
              className={`px-2 py-0.5 text-xs font-medium rounded-full ${badgeColor}`}
            >
              {actionLabel}
            </span>
            {/* Context badge */}
            <ContextBadges contexts={assignedContexts} />
          </div>
          {/* Description */}
          {description && (
            <p className="text-xs text-text-secondary mt-0.5 truncate">
              {description}
            </p>
          )}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2">
          {isDeleting ? (
            <>
              <Button
                variant="danger"
                size="sm"
                onClick={() => onConfirmDelete?.(command.id)}
                aria-label="Confirm delete"
              >
                Confirm
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={onCancelDelete}
                aria-label="Cancel delete"
              >
                Cancel
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onEdit(command)}
                aria-label={`Edit ${command.trigger}`}
              >
                <Pencil className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onDelete(command.id)}
                aria-label={`Delete ${command.trigger}`}
              >
                <X className="h-4 w-4" />
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
