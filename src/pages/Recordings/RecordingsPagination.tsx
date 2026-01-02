import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "../../components/ui";

export interface RecordingsPaginationProps {
  currentPage: number;
  totalPages: number;
  totalCount: number;
  pageSize: number;
  recordingsOnPage: number;
  hasMore: boolean;
  onPreviousPage: () => void;
  onNextPage: () => void;
}

/**
 * Pagination controls for recordings list.
 */
export function RecordingsPagination({
  currentPage,
  totalPages,
  totalCount,
  pageSize,
  recordingsOnPage,
  hasMore,
  onPreviousPage,
  onNextPage,
}: RecordingsPaginationProps) {
  const offset = currentPage * pageSize;

  return (
    <div className="flex items-center justify-between pt-4">
      <p className="text-sm text-text-secondary">
        Showing {offset + 1}-{Math.min(offset + recordingsOnPage, totalCount)} of {totalCount}{" "}
        recordings
      </p>
      <div className="flex items-center gap-2">
        <Button
          variant="secondary"
          size="sm"
          onClick={onPreviousPage}
          disabled={currentPage === 0}
          aria-label="Previous page"
        >
          <ChevronLeft className="h-4 w-4" />
          Previous
        </Button>
        <span className="text-sm text-text-secondary px-2">
          Page {currentPage + 1} of {totalPages}
        </span>
        <Button
          variant="secondary"
          size="sm"
          onClick={onNextPage}
          disabled={!hasMore}
          aria-label="Next page"
        >
          Next
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
