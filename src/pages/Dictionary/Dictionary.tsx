import { useCallback } from "react";
import { Search, Book, Plus } from "lucide-react";
import { Card, CardContent, Button, Input } from "../../components/ui";
import { DictionaryProvider, useDictionaryContext } from "./DictionaryContext";
import { AddEntryForm } from "./AddEntryForm";
import { EntryItem } from "./EntryItem";

export interface DictionaryProps {
  /** Navigate to another page */
  onNavigate?: (page: string) => void;
}

function DictionaryEmptyState({ onAddFocus }: { onAddFocus: () => void }) {
  return (
    <Card className="text-center py-12">
      <CardContent className="flex flex-col items-center gap-4">
        <div className="w-16 h-16 rounded-full bg-heycat-orange/10 flex items-center justify-center">
          <Book className="h-8 w-8 text-heycat-orange" />
        </div>
        <div>
          <h3 className="text-lg font-medium text-text-primary">No dictionary entries yet</h3>
          <p className="text-sm text-text-secondary mt-1">
            Add your first text expansion to get started
          </p>
        </div>
        <Button onClick={onAddFocus}>
          <Plus className="h-4 w-4" />
          Add Entry
        </Button>
      </CardContent>
    </Card>
  );
}

function DictionaryContent() {
  const {
    entryList,
    filteredEntries,
    searchQuery,
    setSearchQuery,
    isLoading,
    isError,
    error,
    refetch,
  } = useDictionaryContext();

  const handleFocusAdd = useCallback(() => {
    const triggerInput = document.querySelector(
      'input[aria-label="Trigger phrase"]'
    ) as HTMLInputElement | null;
    triggerInput?.focus();
  }, []);

  if (isLoading) {
    return (
      <div className="p-6">
        <div className="text-text-secondary" role="status">
          Loading dictionary...
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
              {error?.message ?? "Failed to load dictionary"}
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
        <h1 className="text-2xl font-semibold text-text-primary">Dictionary</h1>
        <p className="text-text-secondary mt-1">
          Create text expansions to speed up your typing.
        </p>
      </header>

      {/* Add Entry Form */}
      <AddEntryForm />

      {/* Search Bar */}
      {entryList.length > 0 && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <Input
            type="text"
            placeholder="Search entries..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
            aria-label="Search dictionary entries"
          />
        </div>
      )}

      {/* Entry List or Empty State */}
      {entryList.length === 0 ? (
        <DictionaryEmptyState onAddFocus={handleFocusAdd} />
      ) : filteredEntries.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">No entries match "{searchQuery}"</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2" role="list" aria-label="Dictionary entries">
          {filteredEntries.map((entry) => (
            <EntryItem key={entry.id} entry={entry} />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Dictionary page component.
 * Provides context for all child components.
 */
export function Dictionary(_props: DictionaryProps) {
  return (
    <DictionaryProvider>
      <DictionaryContent />
    </DictionaryProvider>
  );
}
