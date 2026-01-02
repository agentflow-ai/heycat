import { Search } from "lucide-react";
import { Input, Select, SelectItem } from "../../components/ui";
import type { FilterOption, SortOption } from "../../hooks/useRecordingsFilter";

export interface RecordingsFiltersProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  filterOption: FilterOption;
  onFilterChange: (option: FilterOption) => void;
  sortOption: SortOption;
  onSortChange: (option: SortOption) => void;
}

/**
 * Search, filter, and sort controls for recordings.
 */
export function RecordingsFilters({
  searchQuery,
  onSearchChange,
  filterOption,
  onFilterChange,
  sortOption,
  onSortChange,
}: RecordingsFiltersProps) {
  return (
    <div className="flex flex-col sm:flex-row gap-3">
      {/* Search Input */}
      <div className="relative flex-1">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
        <Input
          type="text"
          placeholder="Search recordings..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-10"
          aria-label="Search recordings"
        />
      </div>

      {/* Filter Dropdown */}
      <div className="w-full sm:w-40">
        <Select
          value={filterOption}
          onValueChange={(value) => onFilterChange(value as FilterOption)}
          placeholder="Filter"
        >
          <SelectItem value="all">All</SelectItem>
          <SelectItem value="transcribed">Transcribed</SelectItem>
          <SelectItem value="pending">Pending</SelectItem>
        </Select>
      </div>

      {/* Sort Dropdown */}
      <div className="w-full sm:w-40">
        <Select
          value={sortOption}
          onValueChange={(value) => onSortChange(value as SortOption)}
          placeholder="Sort by"
        >
          <SelectItem value="newest">Newest</SelectItem>
          <SelectItem value="oldest">Oldest</SelectItem>
          <SelectItem value="longest">Longest</SelectItem>
          <SelectItem value="shortest">Shortest</SelectItem>
        </Select>
      </div>
    </div>
  );
}
