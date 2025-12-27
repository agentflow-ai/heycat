import {
  LayoutDashboard,
  Mic,
  MessageSquare,
  Settings,
  Book,
  Layers,
  type LucideIcon,
} from "lucide-react";

export interface NavItem {
  id: string;
  label: string;
  icon: "LayoutDashboard" | "Mic" | "MessageSquare" | "Settings" | "Book" | "Layers";
}

export interface SidebarProps {
  items: NavItem[];
  activeItemId?: string;
  onItemClick?: (itemId: string) => void;
}

const iconMap: Record<NavItem["icon"], LucideIcon> = {
  LayoutDashboard,
  Mic,
  MessageSquare,
  Settings,
  Book,
  Layers,
};

export function Sidebar({ items, activeItemId, onItemClick }: SidebarProps) {
  return (
    <aside
      className="
        w-[220px] shrink-0
        bg-surface-elevated
        border-r border-border
      "
      role="navigation"
      aria-label="Main navigation"
    >
      <nav className="flex flex-col gap-1 p-3">
        {items.map((item) => {
          const Icon = iconMap[item.icon];
          const isActive = item.id === activeItemId;

          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onItemClick?.(item.id)}
              className={`
                flex items-center gap-3 px-3 py-2.5
                text-sm font-medium
                rounded-[var(--radius-md)]
                transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
                ${
                  isActive
                    ? "bg-heycat-orange/20 text-text-primary"
                    : "text-text-secondary hover:bg-heycat-orange/10 hover:text-text-primary"
                }
              `}
              aria-current={isActive ? "page" : undefined}
            >
              <Icon
                className={`w-5 h-5 ${isActive ? "text-heycat-orange" : ""}`}
                aria-hidden="true"
              />
              <span>{item.label}</span>
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
