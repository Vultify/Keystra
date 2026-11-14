import { useEffect, useState } from "react";

import { getKeyboardStatus, onKeyboardStatus, type KeyboardStatus } from "@/lib/keyboard";
import { logError } from "@/lib/logging";

/** Live keyboard connection state. `null` until the first answer from the worker. */
export function useKeyboardStatus(): KeyboardStatus | null {
  const [status, setStatus] = useState<KeyboardStatus | null>(null);

  useEffect(() => {
    let active = true;
    let heardEvent = false;

    // Listen first, then ask, so a change during startup isn't lost or overwritten.
    const unlisten = onKeyboardStatus((next) => {
      heardEvent = true;
      if (active) setStatus(next);
    });
    getKeyboardStatus()
      .then((current) => {
        if (active && !heardEvent) setStatus(current);
      })
      .catch((err) => logError("could not read keyboard status", err));

    return () => {
      active = false;
      unlisten.then((stop) => stop()).catch(() => {});
    };
  }, []);

  return status;
}
