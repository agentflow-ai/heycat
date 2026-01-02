import { Button } from "../../../components/ui";

export interface ShortcutRecordButtonProps {
  /** Whether currently recording */
  isRecording: boolean;
  /** Handler to start recording */
  onStartRecording: () => Promise<void>;
}

/**
 * Button for starting keyboard shortcut recording.
 */
export function ShortcutRecordButton({
  isRecording,
  onStartRecording,
}: ShortcutRecordButtonProps) {
  return (
    <Button
      variant="secondary"
      onClick={onStartRecording}
      disabled={isRecording}
    >
      {isRecording ? "Recording..." : "Record New Shortcut"}
    </Button>
  );
}
