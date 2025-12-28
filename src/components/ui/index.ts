// Base UI Components - HeyCat Design System
// Source of Truth: ui.md Part 3: Component Library

// Buttons (ui.md 3.1)
export { Button } from "./Button";
export type { ButtonProps, ButtonVariant, ButtonSize } from "./Button";

// Cards (ui.md 3.2)
export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "./Card";
export type {
  CardProps,
  CardVariant,
  CardHeaderProps,
  CardTitleProps,
  CardDescriptionProps,
  CardContentProps,
  CardFooterProps,
} from "./Card";

// Inputs (ui.md 3.3)
export { Input, Textarea, Label, FormField } from "./Input";
export type {
  InputProps,
  TextareaProps,
  LabelProps,
  FormFieldProps,
} from "./Input";

// Select/Dropdown (ui.md 3.3)
export { Select, SelectItem, SelectGroup, SelectSeparator } from "./Select";
export type { SelectProps, SelectItemProps, SelectGroupProps } from "./Select";

// Toggle Switch (ui.md 3.3)
export { Toggle, LabeledToggle } from "./Toggle";
export type { ToggleProps, LabeledToggleProps } from "./Toggle";

// Status Indicators (ui.md 3.4)
export { StatusIndicator, RecordingDot } from "./StatusIndicator";
export type {
  StatusIndicatorProps,
  StatusIndicatorVariant,
  RecordingDotProps,
} from "./StatusIndicator";

// Status Pill (ui.md 2.2, 3.4, 5.3)
export { StatusPill, AutoTimerStatusPill } from "./StatusPill";
export type {
  StatusPillProps,
  StatusPillStatus,
  AutoTimerStatusPillProps,
} from "./StatusPill";

// Connected Status Pill (uses app state hooks)
export { ConnectedStatusPill } from "./ConnectedStatusPill";
export type { ConnectedStatusPillProps } from "./ConnectedStatusPill";

// Audio Level Meter (ui.md 3.4)
export { AudioLevelMeter, MiniAudioMeter } from "./AudioLevelMeter";
export type { AudioLevelMeterProps, MiniAudioMeterProps } from "./AudioLevelMeter";

// Combobox/Autocomplete (custom component)
export { Combobox } from "./Combobox";
export type { ComboboxProps, ComboboxOption } from "./Combobox";

// MultiSelect (custom component)
export { MultiSelect } from "./MultiSelect";
export type { MultiSelectProps, MultiSelectOption } from "./MultiSelect";
