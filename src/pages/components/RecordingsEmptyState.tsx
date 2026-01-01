import { Mic } from "lucide-react";
import { Card, CardContent } from "../../components/ui";

export function RecordingsEmptyState() {
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
            No recordings yet
          </h3>
          <p className="text-sm text-text-secondary mt-1">
            Press ⌘⇧R or say "Hey Cat" to start
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
