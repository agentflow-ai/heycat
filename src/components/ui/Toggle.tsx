import * as SwitchPrimitive from "@radix-ui/react-switch";
import { forwardRef } from "react";

export interface ToggleProps {
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  id?: string;
  name?: string;
  value?: string;
  required?: boolean;
}

export const Toggle = forwardRef<HTMLButtonElement, ToggleProps>(
  ({ ...props }, ref) => (
    <SwitchPrimitive.Root
      ref={ref}
      className="
        relative
        inline-flex items-center
        h-6 w-11
        shrink-0
        cursor-pointer
        rounded-full
        border-2 border-transparent
        bg-neutral-300
        transition-colors duration-[var(--duration-normal)] ease-[var(--ease-default)]
        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-heycat-teal focus-visible:ring-offset-2
        disabled:cursor-not-allowed disabled:opacity-50
        data-[state=checked]:bg-heycat-orange
      "
      {...props}
    >
      <SwitchPrimitive.Thumb
        className="
          pointer-events-none
          block
          h-5 w-5
          rounded-full
          bg-white
          shadow-md
          ring-0
          transition-transform duration-[var(--duration-normal)] ease-[var(--ease-bounce)]
          data-[state=checked]:translate-x-5
          data-[state=unchecked]:translate-x-0
        "
      />
    </SwitchPrimitive.Root>
  )
);

Toggle.displayName = "Toggle";

// Labeled Toggle for convenience
export interface LabeledToggleProps extends ToggleProps {
  label: string;
  description?: string;
}

export const LabeledToggle = forwardRef<HTMLButtonElement, LabeledToggleProps>(
  ({ label, description, id, ...props }, ref) => {
    const toggleId = id || `toggle-${label.toLowerCase().replace(/\s+/g, "-")}`;

    return (
      <div className="flex items-center justify-between gap-4">
        <div className="flex flex-col">
          <label
            htmlFor={toggleId}
            className="text-sm font-medium text-text-primary cursor-pointer"
          >
            {label}
          </label>
          {description && (
            <span className="text-xs text-text-secondary mt-0.5">
              {description}
            </span>
          )}
        </div>
        <Toggle ref={ref} id={toggleId} {...props} />
      </div>
    );
  }
);

LabeledToggle.displayName = "LabeledToggle";
