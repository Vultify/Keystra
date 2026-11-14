import { useEffect, useState } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { logError } from "@/lib/logging";

/**
 * A value the Rust side owns and announces with an event, `null` until the first answer. Pass
 * module-level functions so the effect runs once.
 */
export function useLiveValue<T>(
  load: () => Promise<T>,
  subscribe: (handler: (value: T) => void) => Promise<UnlistenFn>,
  errorMessage: string,
): T | null {
  const [value, setValue] = useState<T | null>(null);

  useEffect(() => {
    let active = true;
    let heardEvent = false;

    // Listen first, then ask, so a change during startup isn't lost or overwritten.
    const unlisten = subscribe((next) => {
      heardEvent = true;
      if (active) setValue(next);
    });
    load()
      .then((current) => {
        if (active && !heardEvent) setValue(current);
      })
      .catch((err) => logError(errorMessage, err));

    return () => {
      active = false;
      unlisten.then((stop) => stop()).catch(() => {});
    };
  }, [load, subscribe, errorMessage]);

  return value;
}
