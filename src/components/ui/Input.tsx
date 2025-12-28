import { type InputHTMLAttributes, forwardRef } from "react";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ error = false, className = "", ...props }, ref) => {
    const baseStyles = `
      w-full
      bg-surface
      border rounded-[var(--radius-sm)]
      px-3.5 py-2.5
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
      <input
        ref={ref}
        className={`${baseStyles} ${borderColor} ${className}`}
        {...props}
      />
    );
  }
);

Input.displayName = "Input";

// Textarea variant
export interface TextareaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ error = false, className = "", ...props }, ref) => {
    const baseStyles = `
      w-full
      bg-surface
      border rounded-[var(--radius-sm)]
      px-3.5 py-2.5
      text-base text-text-primary
      placeholder:text-text-secondary
      transition-all duration-[var(--duration-fast)] ease-[var(--ease-default)]
      focus:outline-none focus:border-heycat-teal focus:ring-2 focus:ring-heycat-teal/10
      disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-text-secondary/10
      resize-y min-h-[80px]
    `;

    const borderColor = error
      ? "border-error focus:border-error focus:ring-error/10"
      : "border-border";

    return (
      <textarea
        ref={ref}
        className={`${baseStyles} ${borderColor} ${className}`}
        {...props}
      />
    );
  }
);

Textarea.displayName = "Textarea";

// Label component
export interface LabelProps
  extends React.LabelHTMLAttributes<HTMLLabelElement> {
  required?: boolean;
}

export const Label = forwardRef<HTMLLabelElement, LabelProps>(
  ({ required, className = "", children, ...props }, ref) => (
    <label
      ref={ref}
      className={`block text-sm font-medium text-text-primary mb-1.5 ${className}`}
      {...props}
    >
      {children}
      {required && <span className="text-error ml-0.5">*</span>}
    </label>
  )
);

Label.displayName = "Label";

// FormField wrapper
export interface FormFieldProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Label text to display above the field */
  label?: string;
  /** Help text to display below the label */
  help?: string;
  /** Error message to display */
  error?: string;
  /** Whether the field is required */
  required?: boolean;
}

export const FormField = forwardRef<HTMLDivElement, FormFieldProps>(
  ({ label, help, error, required, className = "", children, ...props }, ref) => (
    <div ref={ref} className={`mb-4 ${className}`} {...props}>
      {label && (
        <Label required={required}>
          {label}
        </Label>
      )}
      {help && (
        <p className="text-xs text-text-secondary mb-1.5">{help}</p>
      )}
      {children}
      {error && (
        <p className="mt-1.5 text-sm text-error" role="alert">
          {error}
        </p>
      )}
    </div>
  )
);

FormField.displayName = "FormField";
