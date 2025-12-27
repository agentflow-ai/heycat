import {
  useState,
  useRef,
  useEffect,
  useCallback,
  type KeyboardEvent,
} from "react";
import { ChevronDown, Check, X } from "lucide-react";

export interface MultiSelectOption {
  /** Value to use when selected */
  value: string;
  /** Display label for the option */
  label: string;
  /** Optional secondary text */
  description?: string;
}

export interface MultiSelectProps {
  /** Currently selected values */
  selected: string[];
  /** Callback when selection changes */
  onChange: (selected: string[]) => void;
  /** Available options */
  options: MultiSelectOption[];
  /** Placeholder text when nothing is selected */
  placeholder?: string;
  /** Whether the input is disabled */
  disabled?: boolean;
  /** Aria label for accessibility */
  "aria-label"?: string;
  /** Additional class names */
  className?: string;
}

export function MultiSelect({
  selected,
  onChange,
  options,
  placeholder = "Select items...",
  disabled = false,
  "aria-label": ariaLabel,
  className = "",
}: MultiSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const [searchTerm, setSearchTerm] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  // Filter options based on search term
  const filteredOptions = options.filter((option) => {
    const term = searchTerm.toLowerCase();
    return (
      option.label.toLowerCase().includes(term) ||
      (option.description?.toLowerCase().includes(term) ?? false)
    );
  });

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setHighlightedIndex(-1);
        setSearchTerm("");
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Scroll highlighted item into view
  useEffect(() => {
    if (highlightedIndex >= 0 && listRef.current) {
      const highlightedItem = listRef.current.children[
        highlightedIndex
      ] as HTMLElement;
      highlightedItem?.scrollIntoView?.({ block: "nearest" });
    }
  }, [highlightedIndex]);

  const handleToggleOption = useCallback(
    (value: string) => {
      if (selected.includes(value)) {
        onChange(selected.filter((v) => v !== value));
      } else {
        onChange([...selected, value]);
      }
    },
    [selected, onChange]
  );

  const handleRemoveSelected = useCallback(
    (value: string, e: React.MouseEvent) => {
      e.stopPropagation();
      onChange(selected.filter((v) => v !== value));
    },
    [selected, onChange]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (!isOpen && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
        setIsOpen(true);
        e.preventDefault();
        return;
      }

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev < filteredOptions.length - 1 ? prev + 1 : 0
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev > 0 ? prev - 1 : filteredOptions.length - 1
          );
          break;
        case "Enter":
          e.preventDefault();
          if (highlightedIndex >= 0 && filteredOptions[highlightedIndex]) {
            handleToggleOption(filteredOptions[highlightedIndex].value);
          }
          break;
        case "Escape":
          setIsOpen(false);
          setHighlightedIndex(-1);
          setSearchTerm("");
          break;
        case "Backspace":
          if (searchTerm === "" && selected.length > 0) {
            // Remove last selected item
            onChange(selected.slice(0, -1));
          }
          break;
      }
    },
    [isOpen, filteredOptions, highlightedIndex, handleToggleOption, searchTerm, selected, onChange]
  );

  const handleFocus = useCallback(() => {
    setIsOpen(true);
  }, []);

  const handleToggleDropdown = useCallback(() => {
    if (!disabled) {
      setIsOpen((prev) => !prev);
      inputRef.current?.focus();
    }
  }, [disabled]);

  // Get labels for selected values
  const selectedOptions = options.filter((opt) => selected.includes(opt.value));

  return (
    <div ref={containerRef} className={`relative ${className}`}>
      <div
        className={`
          flex flex-wrap items-center gap-1 min-h-[42px]
          bg-surface
          border border-border rounded-[var(--radius-sm)]
          px-2 py-1.5
          transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
          focus-within:border-heycat-teal focus-within:ring-2 focus-within:ring-heycat-teal/10
          ${disabled ? "opacity-50 cursor-not-allowed bg-text-secondary/10" : "cursor-text"}
        `}
        onClick={() => !disabled && inputRef.current?.focus()}
      >
        {/* Selected items as tags */}
        {selectedOptions.map((option) => (
          <span
            key={option.value}
            className="inline-flex items-center gap-1 px-2 py-0.5 text-sm bg-heycat-orange/10 text-text-primary rounded"
          >
            {option.label}
            <button
              type="button"
              onClick={(e) => handleRemoveSelected(option.value, e)}
              className="hover:text-error transition-colors"
              aria-label={`Remove ${option.label}`}
              disabled={disabled}
            >
              <X className="h-3 w-3" />
            </button>
          </span>
        ))}

        {/* Search input */}
        <input
          ref={inputRef}
          type="text"
          value={searchTerm}
          onChange={(e) => {
            setSearchTerm(e.target.value);
            setIsOpen(true);
            setHighlightedIndex(-1);
          }}
          onKeyDown={handleKeyDown}
          onFocus={handleFocus}
          placeholder={selected.length === 0 ? placeholder : ""}
          disabled={disabled}
          aria-label={ariaLabel}
          aria-expanded={isOpen}
          aria-haspopup="listbox"
          role="combobox"
          className="flex-1 min-w-[60px] bg-transparent border-none outline-none text-sm text-text-primary placeholder:text-text-secondary"
        />

        {/* Dropdown toggle button */}
        <button
          type="button"
          onClick={handleToggleDropdown}
          disabled={disabled}
          tabIndex={-1}
          className="p-1 text-text-secondary hover:text-text-primary transition-colors shrink-0"
          aria-label="Toggle dropdown"
        >
          <ChevronDown
            className={`h-4 w-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
          />
        </button>
      </div>

      {/* Dropdown list */}
      {isOpen && (
        <ul
          ref={listRef}
          role="listbox"
          aria-multiselectable="true"
          className="
            absolute z-50 w-full mt-1
            max-h-60 overflow-auto
            bg-surface
            border border-border rounded-[var(--radius-md)]
            shadow-lg
            py-1
          "
        >
          {filteredOptions.length === 0 ? (
            <li className="px-3.5 py-2 text-sm text-text-secondary">
              {options.length === 0 ? "No options available" : "No matching options"}
            </li>
          ) : (
            filteredOptions.map((option, index) => {
              const isSelected = selected.includes(option.value);
              return (
                <li
                  key={option.value}
                  role="option"
                  aria-selected={isSelected}
                  className={`
                    flex items-center gap-2 px-3.5 py-2 cursor-pointer
                    transition-colors duration-[var(--duration-fast)]
                    ${
                      highlightedIndex === index
                        ? "bg-heycat-orange/10"
                        : "hover:bg-heycat-orange/5"
                    }
                  `}
                  onClick={() => handleToggleOption(option.value)}
                  onMouseEnter={() => setHighlightedIndex(index)}
                >
                  {/* Checkbox indicator */}
                  <span
                    className={`
                      flex items-center justify-center w-4 h-4 rounded border shrink-0
                      ${
                        isSelected
                          ? "bg-heycat-orange border-heycat-orange"
                          : "border-border"
                      }
                    `}
                  >
                    {isSelected && <Check className="h-3 w-3 text-white" />}
                  </span>

                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-text-primary">{option.label}</div>
                    {option.description && (
                      <div className="text-xs text-text-secondary truncate">
                        {option.description}
                      </div>
                    )}
                  </div>
                </li>
              );
            })
          )}
        </ul>
      )}
    </div>
  );
}

MultiSelect.displayName = "MultiSelect";
