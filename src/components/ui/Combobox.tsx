import {
  useState,
  useRef,
  useEffect,
  useCallback,
  type KeyboardEvent,
} from "react";
import { ChevronDown } from "lucide-react";

export interface ComboboxOption {
  /** Display label for the option */
  label: string;
  /** Value to use when selected */
  value: string;
  /** Optional secondary text (e.g., bundle ID) */
  description?: string;
}

export interface ComboboxProps {
  /** Current value (controlled) */
  value: string;
  /** Callback when value changes */
  onChange: (value: string) => void;
  /** Callback when an option is selected (provides full option data) */
  onSelect?: (option: ComboboxOption) => void;
  /** Available options to filter from */
  options: ComboboxOption[];
  /** Placeholder text */
  placeholder?: string;
  /** Whether the input is disabled */
  disabled?: boolean;
  /** Error state */
  error?: boolean;
  /** Aria label for accessibility */
  "aria-label"?: string;
  /** Additional class names */
  className?: string;
}

export function Combobox({
  value,
  onChange,
  onSelect,
  options,
  placeholder,
  disabled = false,
  error = false,
  "aria-label": ariaLabel,
  className = "",
}: ComboboxProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  // Filter options based on input value
  const filteredOptions = options.filter((option) => {
    const searchTerm = value.toLowerCase();
    return (
      option.label.toLowerCase().includes(searchTerm) ||
      (option.description?.toLowerCase().includes(searchTerm) ?? false)
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
      // scrollIntoView may not be available in all environments (e.g., jsdom)
      highlightedItem?.scrollIntoView?.({ block: "nearest" });
    }
  }, [highlightedIndex]);

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(e.target.value);
      setIsOpen(true);
      setHighlightedIndex(-1);
    },
    [onChange]
  );

  const handleSelectOption = useCallback(
    (option: ComboboxOption) => {
      onChange(option.value);
      onSelect?.(option);
      setIsOpen(false);
      setHighlightedIndex(-1);
      inputRef.current?.focus();
    },
    [onChange, onSelect]
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
            handleSelectOption(filteredOptions[highlightedIndex]);
          }
          break;
        case "Escape":
          setIsOpen(false);
          setHighlightedIndex(-1);
          break;
      }
    },
    [isOpen, filteredOptions, highlightedIndex, handleSelectOption]
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

  const baseStyles = `
    w-full
    bg-surface
    border rounded-[var(--radius-sm)]
    px-3.5 py-2.5 pr-10
    text-base text-text-primary
    placeholder:text-text-secondary
    transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
    focus:outline-none focus:border-heycat-teal focus:ring-2 focus:ring-heycat-teal/10
    disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-text-secondary/10
  `;

  const borderColor = error
    ? "border-error focus:border-error focus:ring-error/10"
    : "border-border";

  return (
    <div ref={containerRef} className={`relative ${className}`}>
      <div className="relative">
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          onFocus={handleFocus}
          placeholder={placeholder}
          disabled={disabled}
          aria-label={ariaLabel}
          aria-expanded={isOpen}
          aria-haspopup="listbox"
          aria-autocomplete="list"
          role="combobox"
          className={`${baseStyles} ${borderColor}`}
        />
        <button
          type="button"
          onClick={handleToggleDropdown}
          disabled={disabled}
          tabIndex={-1}
          className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-text-secondary hover:text-text-primary transition-colors"
          aria-label="Toggle dropdown"
        >
          <ChevronDown
            className={`h-4 w-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
          />
        </button>
      </div>

      {isOpen && filteredOptions.length > 0 && (
        <ul
          ref={listRef}
          role="listbox"
          className="
            absolute z-50 w-full mt-1
            max-h-60 overflow-auto
            bg-surface
            border border-border rounded-[var(--radius-md)]
            shadow-lg
            py-1
          "
        >
          {filteredOptions.map((option, index) => (
            <li
              key={option.value}
              role="option"
              aria-selected={highlightedIndex === index}
              className={`
                px-3.5 py-2 cursor-pointer
                transition-colors duration-[var(--duration-fast)]
                ${
                  highlightedIndex === index
                    ? "bg-heycat-orange/10"
                    : "hover:bg-heycat-orange/5"
                }
              `}
              onClick={() => handleSelectOption(option)}
              onMouseEnter={() => setHighlightedIndex(index)}
            >
              <div className="text-sm text-text-primary">{option.label}</div>
              {option.description && (
                <div className="text-xs text-text-secondary truncate">
                  {option.description}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}

      {isOpen && value && filteredOptions.length === 0 && (
        <div className="absolute z-50 w-full mt-1 p-3 bg-surface border border-border rounded-[var(--radius-md)] shadow-lg">
          <div className="text-sm text-text-secondary">
            No matching applications. Custom value will be used.
          </div>
        </div>
      )}
    </div>
  );
}

Combobox.displayName = "Combobox";
