import type { LucideIcon } from "lucide-react";
import {
  Mic,
  MicOff,
  LayoutDashboard,
  ListMusic,
  MessageSquare,
  Settings,
  SpeakerIcon,
  Download,
  Keyboard,
  Info,
} from "lucide-react";

export type CommandCategory = "actions" | "navigation" | "settings" | "help";

export interface Command {
  id: string;
  label: string;
  icon: LucideIcon;
  shortcut?: string;
  category: CommandCategory;
  keywords?: string[];
}

/**
 * Command registry for the command palette.
 * Actions are executed via callbacks passed to CommandPalette.
 */
export const commands: Command[] = [
  // Actions
  {
    id: "start-recording",
    label: "Start Recording",
    icon: Mic,
    shortcut: "⌘⇧R",
    category: "actions",
    keywords: ["record", "mic", "audio", "capture"],
  },
  {
    id: "stop-recording",
    label: "Stop Recording",
    icon: MicOff,
    shortcut: "Esc",
    category: "actions",
    keywords: ["record", "stop", "end"],
  },

  // Navigation
  {
    id: "go-dashboard",
    label: "Go to Dashboard",
    icon: LayoutDashboard,
    category: "navigation",
    keywords: ["home", "main", "overview"],
  },
  {
    id: "go-recordings",
    label: "Go to Recordings",
    icon: ListMusic,
    category: "navigation",
    keywords: ["history", "audio", "files"],
  },
  {
    id: "go-commands",
    label: "Go to Commands",
    icon: MessageSquare,
    category: "navigation",
    keywords: ["voice", "commands", "phrases"],
  },
  {
    id: "go-settings",
    label: "Go to Settings",
    icon: Settings,
    shortcut: "⌘,",
    category: "navigation",
    keywords: ["preferences", "config", "options"],
  },

  // Settings
  {
    id: "change-audio-device",
    label: "Change Audio Device",
    icon: SpeakerIcon,
    category: "settings",
    keywords: ["mic", "microphone", "input", "sound"],
  },
  {
    id: "download-model",
    label: "Download Model",
    icon: Download,
    category: "settings",
    keywords: ["parakeet", "transcription", "ml", "ai"],
  },

  // Help
  {
    id: "view-shortcuts",
    label: "View Shortcuts",
    icon: Keyboard,
    category: "help",
    keywords: ["keys", "hotkeys", "keyboard"],
  },
  {
    id: "about-heycat",
    label: "About HeyCat",
    icon: Info,
    category: "help",
    keywords: ["version", "info", "app"],
  },
];

/**
 * Get commands grouped by category
 */
export function getCommandsByCategory(): Record<CommandCategory, Command[]> {
  return {
    actions: commands.filter((c) => c.category === "actions"),
    navigation: commands.filter((c) => c.category === "navigation"),
    settings: commands.filter((c) => c.category === "settings"),
    help: commands.filter((c) => c.category === "help"),
  };
}

/**
 * Category display names
 */
export const categoryLabels: Record<CommandCategory, string> = {
  actions: "Actions",
  navigation: "Navigation",
  settings: "Settings",
  help: "Help",
};

/**
 * Simple fuzzy search for commands
 * Matches against label and keywords
 */
export function filterCommands(query: string): Command[] {
  if (!query.trim()) {
    return commands;
  }

  const lowerQuery = query.toLowerCase();

  return commands.filter((command) => {
    // Match against label
    if (command.label.toLowerCase().includes(lowerQuery)) {
      return true;
    }

    // Match against keywords
    if (command.keywords?.some((kw) => kw.toLowerCase().includes(lowerQuery))) {
      return true;
    }

    return false;
  });
}
