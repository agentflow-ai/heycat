import { useState, useCallback } from "react";
import { Search, Layers, Plus, ChevronDown, ChevronUp, HelpCircle } from "lucide-react";
import { Card, CardContent, Button, Input } from "../../components/ui";
import { WindowContextsProvider, useWindowContextsContext } from "./WindowContextsContext";
import { AddContextForm } from "./AddContextForm";
import { ContextItem } from "./ContextItem";

export interface WindowContextsProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

function EmptyState({ onAddFocus }: { onAddFocus: () => void }) {
  return (
    <Card className="text-center py-12">
      <CardContent className="flex flex-col items-center gap-4">
        <div className="w-16 h-16 rounded-full bg-heycat-orange/10 flex items-center justify-center">
          <Layers className="h-8 w-8 text-heycat-orange" />
        </div>
        <div>
          <h3 className="text-lg font-medium text-text-primary">No window contexts yet</h3>
          <p className="text-sm text-text-secondary mt-1">
            Create contexts to customize commands and dictionary per app
          </p>
        </div>
        <Button onClick={onAddFocus}>
          <Plus className="h-4 w-4" />
          Add Context
        </Button>
      </CardContent>
    </Card>
  );
}

const DIAGRAM_COLLAPSED_KEY = "heycat-context-diagram-collapsed";

function ModeExplanationDiagram() {
  const [isCollapsed, setIsCollapsed] = useState(() => {
    const stored = localStorage.getItem(DIAGRAM_COLLAPSED_KEY);
    return stored === "true";
  });

  const toggleCollapsed = () => {
    const newValue = !isCollapsed;
    setIsCollapsed(newValue);
    localStorage.setItem(DIAGRAM_COLLAPSED_KEY, String(newValue));
  };

  return (
    <Card className="bg-neutral-50 dark:bg-neutral-900 border-neutral-200 dark:border-neutral-800">
      <button
        onClick={toggleCollapsed}
        className="w-full flex items-center justify-between p-3 text-left hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors rounded-t-lg"
        aria-expanded={!isCollapsed}
      >
        <span className="flex items-center gap-2 text-sm font-medium text-text-secondary">
          <HelpCircle className="h-4 w-4" />
          What does "Context Only" do?
        </span>
        {isCollapsed ? (
          <ChevronDown className="h-4 w-4 text-text-tertiary" />
        ) : (
          <ChevronUp className="h-4 w-4 text-text-tertiary" />
        )}
      </button>
      {!isCollapsed && (
        <CardContent className="pt-0 pb-3 px-3">
          <p className="text-sm text-text-secondary">
            By default, contexts inherit all global commands and dictionary entries.
            Turn on <strong>"Context Only"</strong> to hide global items and use only what's
            assigned to this specific context.
          </p>
        </CardContent>
      )}
    </Card>
  );
}

function WindowContextsContent() {
  const {
    contextList,
    filteredContexts,
    searchQuery,
    setSearchQuery,
    isLoading,
    isError,
    error,
    refetch,
  } = useWindowContextsContext();

  const handleFocusAdd = useCallback(() => {
    const nameInput = document.querySelector(
      'input[aria-label="Context name"]'
    ) as HTMLInputElement | null;
    nameInput?.focus();
  }, []);

  if (isLoading) {
    return (
      <div className="p-6">
        <div className="text-text-secondary" role="status">
          Loading contexts...
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="p-6">
        <Card className="border-error">
          <CardContent>
            <div className="text-error" role="alert">
              {error?.message ?? "Failed to load contexts"}
            </div>
            <Button onClick={() => refetch()} className="mt-4">
              Retry
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">Window Contexts</h1>
        <p className="text-text-secondary mt-1">
          Customize commands and dictionary per application.
        </p>
      </header>

      {/* Mode Explanation */}
      <ModeExplanationDiagram />

      {/* Add Context Form */}
      <AddContextForm />

      {/* Search Bar */}
      {contextList.length > 0 && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <Input
            type="text"
            placeholder="Search contexts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
            aria-label="Search window contexts"
          />
        </div>
      )}

      {/* Context List or Empty State */}
      {contextList.length === 0 ? (
        <EmptyState onAddFocus={handleFocusAdd} />
      ) : filteredContexts.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">No contexts match "{searchQuery}"</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2" role="list" aria-label="Window contexts">
          {filteredContexts.map((ctx) => (
            <ContextItem key={ctx.id} context={ctx} />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Window Contexts page component.
 * Provides context for all child components.
 */
export function WindowContexts(_props: WindowContextsProps) {
  return (
    <WindowContextsProvider>
      <WindowContextsContent />
    </WindowContextsProvider>
  );
}
