import { useState, useEffect, useRef, useCallback } from "react";
import { Search } from "lucide-react";
import { Input } from "../ui";
import {
  commands,
  filterCommands,
  getCommandsByCategory,
  categoryLabels,
  type Command,
  type CommandCategory,
} from "./commands";

export interface CommandPaletteProps {
  /** Whether the palette is open */
  isOpen: boolean;
  /** Callback when palette should close */
  onClose: () => void;
  /** Callback when a command is executed */
  onCommandExecute: (commandId: string) => void;
}

const CATEGORY_ORDER: CommandCategory[] = [
  "actions",
  "navigation",
  "settings",
  "help",
];

export function CommandPalette({
  isOpen,
  onClose,
  onCommandExecute,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const filteredCommands = filterCommands(query);

  // Group filtered commands by category (preserving order)
  const groupedCommands = getCommandsByCategory();
  const filteredGrouped = CATEGORY_ORDER.map((category) => ({
    category,
    commands: filteredCommands.filter((c) => c.category === category),
  })).filter((group) => group.commands.length > 0);

  // Flatten for keyboard navigation
  const flatCommands = filteredGrouped.flatMap((g) => g.commands);

  // Reset state when opening
  useEffect(() => {
    if (isOpen) {
      setQuery("");
      setSelectedIndex(0);
      // Focus input after render
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    if (!isOpen || flatCommands.length === 0) return;

    const selectedCommand = flatCommands[selectedIndex];
    if (!selectedCommand) return;

    const itemEl = listRef.current?.querySelector(
      `[data-command-id="${selectedCommand.id}"]`
    );
    // scrollIntoView may not be available in all environments (e.g., jsdom)
    if (itemEl && typeof itemEl.scrollIntoView === "function") {
      itemEl.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex, flatCommands, isOpen]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev < flatCommands.length - 1 ? prev + 1 : prev
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case "Enter":
          e.preventDefault();
          if (flatCommands[selectedIndex]) {
            onCommandExecute(flatCommands[selectedIndex].id);
            onClose();
          }
          break;
        case "Escape":
          e.preventDefault();
          onClose();
          break;
      }
    },
    [flatCommands, selectedIndex, onCommandExecute, onClose]
  );

  // Handle click outside
  const handleBackdropClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        onClose();
      }
    },
    [onClose]
  );

  // Handle command item click
  const handleCommandClick = useCallback(
    (command: Command) => {
      onCommandExecute(command.id);
      onClose();
    },
    [onCommandExecute, onClose]
  );

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex justify-center bg-black/50"
      onClick={handleBackdropClick}
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
    >
      <div
        className="
          w-[560px] mt-[20vh]
          bg-surface rounded-lg shadow-lg
          flex flex-col max-h-[60vh]
          overflow-hidden
        "
        onKeyDown={handleKeyDown}
      >
        {/* Search input */}
        <div className="flex items-center gap-2 px-4 py-3 border-b border-border">
          <Search
            className="w-5 h-5 text-text-secondary shrink-0"
            aria-hidden="true"
          />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search commands..."
            className="
              flex-1 bg-transparent
              text-base text-text-primary
              placeholder:text-text-secondary
              outline-none
            "
            aria-label="Search commands"
            aria-autocomplete="list"
            aria-controls="command-list"
            aria-activedescendant={
              flatCommands[selectedIndex]
                ? `command-${flatCommands[selectedIndex].id}`
                : undefined
            }
          />
        </div>

        {/* Command list */}
        <div
          ref={listRef}
          id="command-list"
          className="flex-1 overflow-y-auto py-2"
          role="listbox"
        >
          {flatCommands.length === 0 ? (
            <div className="px-4 py-8 text-center text-text-secondary">
              No results found
            </div>
          ) : (
            filteredGrouped.map((group) => (
              <div key={group.category}>
                <div className="px-4 py-1.5 text-xs font-medium text-text-secondary uppercase tracking-wide">
                  {categoryLabels[group.category]}
                </div>
                {group.commands.map((command) => {
                  const index = flatCommands.indexOf(command);
                  const isSelected = index === selectedIndex;
                  const Icon = command.icon;

                  return (
                    <div
                      key={command.id}
                      id={`command-${command.id}`}
                      data-command-id={command.id}
                      role="option"
                      aria-selected={isSelected}
                      onClick={() => handleCommandClick(command)}
                      onMouseEnter={() => setSelectedIndex(index)}
                      className={`
                        flex items-center gap-3 px-4 py-2.5 cursor-pointer
                        transition-colors duration-[var(--duration-fast)]
                        ${
                          isSelected
                            ? "bg-heycat-teal/10 text-heycat-teal"
                            : "text-text-primary hover:bg-neutral-100"
                        }
                      `}
                    >
                      <Icon
                        className={`w-4 h-4 shrink-0 ${isSelected ? "text-heycat-teal" : "text-text-secondary"}`}
                        aria-hidden="true"
                      />
                      <span className="flex-1">{command.label}</span>
                      {command.shortcut && (
                        <span
                          className={`
                            text-xs font-mono
                            ${isSelected ? "text-heycat-teal/70" : "text-text-secondary"}
                          `}
                        >
                          {command.shortcut}
                        </span>
                      )}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
