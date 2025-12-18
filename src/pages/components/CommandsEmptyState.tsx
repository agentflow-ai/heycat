import { Plus, Mic } from "lucide-react";
import { Card, CardContent, Button } from "../../components/ui";

export interface CommandsEmptyStateProps {
  onAddCommand: () => void;
}

export function CommandsEmptyState({ onAddCommand }: CommandsEmptyStateProps) {
  return (
    <Card className="text-center py-12">
      <CardContent className="flex flex-col items-center gap-4">
        {/* Icon */}
        <div className="w-16 h-16 rounded-full bg-heycat-orange/10 flex items-center justify-center">
          <Mic className="h-8 w-8 text-heycat-orange" />
        </div>

        {/* Text */}
        <div>
          <h3 className="text-lg font-medium text-text-primary">
            No voice commands yet
          </h3>
          <p className="text-sm text-text-secondary mt-1">
            Create your first command to get started
          </p>
        </div>

        {/* CTA Button */}
        <Button onClick={onAddCommand}>
          <Plus className="h-4 w-4" />
          New Command
        </Button>
      </CardContent>
    </Card>
  );
}
