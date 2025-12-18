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

export function Commands({ onNavigate }: CommandsProps) {
  const { toast } = useToast();
  const [commands, setCommands] = useState<CommandDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  // Modal state
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingCommand, setEditingCommand] = useState<CommandDto | null>(null);

  // Delete confirmation state
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);

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
    parameters: Record<string, string>
  ) => {
    try {
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
        setCommands((prev) => [...prev, newCommand]);
        toast({
          type: "success",
          title: "Command created",
          description: `"${trigger}" has been added.`,
        });
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
        onSave={handleSaveCommand}
      />
    </div>
  );
}
