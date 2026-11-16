import type { ComponentProps } from "react";

import { Badge } from "@/components/ui/badge";
import type { KeyboardStatus } from "@/lib/keyboard";

type BadgeVariant = ComponentProps<typeof Badge>["variant"];

function describe(status: KeyboardStatus | null): { label: string; variant: BadgeVariant } {
  switch (status?.state) {
    case undefined:
      return { label: "Looking for keyboard", variant: "outline" };
    case "connected":
      return { label: status.info.model ?? "Keyboard connected", variant: "default" };
    case "disconnected":
      return { label: "Keyboard not connected", variant: "outline" };
    case "unsupported":
      return { label: "Unsupported device", variant: "destructive" };
    case "error":
      return { label: "Keyboard error", variant: "destructive" };
    case "installedRunning":
      return { label: "Used by installed Keystra", variant: "outline" };
    case "retrying":
      return { label: "Keyboard not answering", variant: "destructive" };
  }
}

// Small connection indicator for the sidebar footer.
export function KeyboardStatusBadge({ status }: { status: KeyboardStatus | null }) {
  const { label, variant } = describe(status);
  return <Badge variant={variant}>{label}</Badge>;
}
