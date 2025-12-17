import { Slot } from "@radix-ui/react-slot";
import { type ButtonHTMLAttributes, forwardRef } from "react";
import { Loader2 } from "lucide-react";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  asChild?: boolean;
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: `
    bg-gradient-to-br from-heycat-orange to-heycat-orange-light
    text-white
    shadow-sm
    hover:shadow-md hover:-translate-y-px
    active:translate-y-0 active:shadow-sm
  `,
  secondary: `
    bg-white
    border border-heycat-orange
    text-heycat-orange
    hover:bg-heycat-cream
  `,
  ghost: `
    bg-transparent
    text-text-secondary
    hover:bg-neutral-100
  `,
  danger: `
    bg-error
    text-white
    shadow-sm
    hover:shadow-md hover:-translate-y-px
    active:translate-y-0 active:shadow-sm
  `,
};

const sizeStyles: Record<ButtonSize, string> = {
  sm: "px-3 py-1.5 text-sm",
  md: "px-5 py-2.5 text-base",
  lg: "px-6 py-3 text-lg",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = "primary",
      size = "md",
      loading = false,
      disabled,
      asChild = false,
      className = "",
      children,
      ...props
    },
    ref
  ) => {
    const Comp = asChild ? Slot : "button";

    const baseStyles = `
      inline-flex items-center justify-center gap-2
      font-medium
      rounded-[var(--radius-sm)]
      transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
      focus:outline-none focus:ring-2 focus:ring-heycat-teal focus:ring-offset-2
      disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none
    `;

    // When asChild is true, Slot expects exactly one child element
    // So we render either spinner+children (regular button) or just children (asChild)
    const content = asChild ? (
      children
    ) : (
      <>
        {loading && (
          <Loader2 className="h-4 w-4 animate-spin" data-testid="button-spinner" />
        )}
        {children}
      </>
    );

    return (
      <Comp
        ref={ref}
        disabled={disabled || loading}
        className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
        {...props}
      >
        {content}
      </Comp>
    );
  }
);

Button.displayName = "Button";
