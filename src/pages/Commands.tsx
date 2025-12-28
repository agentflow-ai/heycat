import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Search } from "lucide-react";
import {
  Card,
  CardContent,
  Button,
  Input,
} from "../components/ui";
import { useToast } from "../components/overlays";
import { CommandItem } from "./components/CommandItem";
import { CommandModal } from "./components/CommandModal";
import { CommandsEmptyState } from "./components/CommandsEmptyState";
import { useWindowContext } from "../hooks/useWindowContext";
import type { WindowContext } from "../types/windowContext";

export interface CommandDto {
  id: string;
  trigger: string;
  action_type: string;
  parameters: Record<string, string>;
  enabled: boolean;
}

export interface CommandsProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

export function Commands(_props: CommandsProps) {
  const { toast } = useToast();
  const { contexts: contextsQuery, updateContext } = useWindowContext();
  const [commands, setCommands] = useState<CommandDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  // Modal state
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingCommand, setEditingCommand] = useState<CommandDto | null>(null);

  // Delete confirmation state
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

  const contextList = contextsQuery.data ?? [];

  // Reverse lookup: which contexts contain each command
  const contextsByCommandId = useMemo(() => {
    const map = new Map<string, WindowContext[]>();
    for (const ctx of contextList) {
      for (const cmdId of ctx.commandIds) {
        const existing = map.get(cmdId) ?? [];
        existing.push(ctx);
        map.set(cmdId, existing);
      }
    }
    return map;
  }, [contextList]);

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

  // Filter commands by search query
  const filteredCommands = useMemo(() => {
    if (!searchQuery.trim()) return commands;
    const query = searchQuery.toLowerCase();
    return commands.filter(
      (cmd) =>
        cmd.trigger.toLowerCase().includes(query) ||
        cmd.action_type.toLowerCase().includes(query)
    );
  }, [commands, searchQuery]);

  const handleAddCommand = () => {
    setEditingCommand(null);
    setIsModalOpen(true);
  };

  const handleEditCommand = (command: CommandDto) => {
    setEditingCommand(command);
    setIsModalOpen(true);
  };

  const handleSaveCommand = async (
    trigger: string,
    actionType: string,
    parameters: Record<string, string>,
    contextIds: string[]
  ) => {
    try {
      let commandId: string;

      if (editingCommand) {
        // Update existing command
        const updatedCommand = await invoke<CommandDto>("update_command", {
          input: {
            id: editingCommand.id,
            trigger,
            action_type: actionType,
            parameters,
            enabled: editingCommand.enabled,
          },
        });
        commandId = updatedCommand.id;
        setCommands((prev) =>
          prev.map((c) => (c.id === editingCommand.id ? updatedCommand : c))
        );
        toast({
          type: "success",
          title: "Command updated",
          description: `"${trigger}" has been updated.`,
        });
      } else {
        // Add new command
        const newCommand = await invoke<CommandDto>("add_command", {
          input: {
            trigger,
            action_type: actionType,
            parameters,
            enabled: true,
          },
        });
        commandId = newCommand.id;
        setCommands((prev) => [...prev, newCommand]);
        toast({
          type: "success",
          title: "Command created",
          description: `"${trigger}" has been added.`,
        });
      }

      // Update window contexts to reflect the new associations
      const previousContextIds = editingCommand
        ? (contextsByCommandId.get(editingCommand.id) ?? []).map((c) => c.id)
        : [];

      // Contexts to add the command to
      const contextsToAdd = contextIds.filter((id) => !previousContextIds.includes(id));
      // Contexts to remove the command from
      const contextsToRemove = previousContextIds.filter((id) => !contextIds.includes(id));

      // Update contexts that should now include this command
      for (const ctxId of contextsToAdd) {
        const ctx = contextList.find((c) => c.id === ctxId);
        if (ctx) {
          await updateContext.mutateAsync({
            id: ctx.id,
            name: ctx.name,
            appName: ctx.matcher.appName,
            titlePattern: ctx.matcher.titlePattern,
            bundleId: ctx.matcher.bundleId,
            commandMode: ctx.commandMode,
            dictionaryMode: ctx.dictionaryMode,
            commandIds: [...ctx.commandIds, commandId],
            dictionaryEntryIds: ctx.dictionaryEntryIds,
            priority: ctx.priority,
            enabled: ctx.enabled,
          });
        }
      }

      // Update contexts that should no longer include this command
      for (const ctxId of contextsToRemove) {
        const ctx = contextList.find((c) => c.id === ctxId);
        if (ctx) {
          await updateContext.mutateAsync({
            id: ctx.id,
            name: ctx.name,
            appName: ctx.matcher.appName,
            titlePattern: ctx.matcher.titlePattern,
            bundleId: ctx.matcher.bundleId,
            commandMode: ctx.commandMode,
            dictionaryMode: ctx.dictionaryMode,
            commandIds: ctx.commandIds.filter((id) => id !== commandId),
            dictionaryEntryIds: ctx.dictionaryEntryIds,
            priority: ctx.priority,
            enabled: ctx.enabled,
          });
        }
      }

      setIsModalOpen(false);
      setEditingCommand(null);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to save command",
        description: e instanceof Error ? e.message : String(e),
      });
      throw e;
    }
  };

  const handleDeleteCommand = async (id: string) => {
    const command = commands.find((c) => c.id === id);
    try {
      await invoke("remove_command", { id });
      setCommands((prev) => prev.filter((c) => c.id !== id));
      setDeleteConfirmId(null);
      toast({
        type: "success",
        title: "Command deleted",
        description: command ? `"${command.trigger}" has been removed.` : "Command removed.",
      });
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to delete command",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };

  const handleToggleEnabled = async (command: CommandDto) => {
    try {
      const updatedCommand = await invoke<CommandDto>("update_command", {
        input: {
          id: command.id,
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
      toast({
        type: "error",
        title: "Failed to toggle command",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };

  const handleModalClose = () => {
    setIsModalOpen(false);
    setEditingCommand(null);
  };

  if (loading) {
    return (
      <div className="p-6">
        <div className="text-text-secondary" role="status">
          Loading commands...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <Card className="border-error">
          <CardContent>
            <div className="text-error" role="alert">
              {error}
            </div>
            <Button onClick={loadCommands} className="mt-4">
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const existingTriggers = commands
    .filter((c) => c.id !== editingCommand?.id)
    .map((c) => c.trigger.toLowerCase());

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-text-primary">
            Voice Commands
          </h1>
          <p className="text-text-secondary mt-1">
            Create custom voice commands to control your Mac.
          </p>
        </div>
        <Button onClick={handleAddCommand}>
          <Plus className="h-4 w-4" />
          New Command
        </Button>
      </header>

      {/* Search Bar */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
        <Input
          type="text"
          placeholder="Search commands..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-10"
          aria-label="Search commands"
        />
      </div>

      {/* Command List */}
      {commands.length === 0 ? (
        <CommandsEmptyState onAddCommand={handleAddCommand} />
      ) : filteredCommands.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">
              No commands match "{searchQuery}"
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2" role="list" aria-label="Voice commands list">
          {filteredCommands.map((command) => (
            <CommandItem
              key={command.id}
              command={command}
              assignedContexts={contextsByCommandId.get(command.id) ?? []}
              onEdit={handleEditCommand}
              onDelete={(id) => setDeleteConfirmId(id)}
              onToggleEnabled={handleToggleEnabled}
              isDeleting={deleteConfirmId === command.id}
              onConfirmDelete={handleDeleteCommand}
              onCancelDelete={() => setDeleteConfirmId(null)}
            />
          ))}
        </div>
      )}

      {/* Command Modal */}
      <CommandModal
        open={isModalOpen}
        onOpenChange={handleModalClose}
        command={editingCommand}
        existingTriggers={existingTriggers}
        contexts={contextList}
        assignedContextIds={
          editingCommand
            ? (contextsByCommandId.get(editingCommand.id) ?? []).map((c) => c.id)
            : []
        }
        onSave={handleSaveCommand}
      />
    </div>
  );
}
