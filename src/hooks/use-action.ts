import { useCallback, useState } from "react";

import { errorText } from "@/lib/errors";
import { logError } from "@/lib/logging";

export type Action = {
  /** Which action is running, by the key it was started with, or `null`. */
  busy: string | null;
  error: string | null;
  /** Runs an action, tracking its busy state and catching its error. Resolves to whether it worked. */
  run: (key: string, action: () => Promise<unknown>) => Promise<boolean>;
};

/** Busy and error state for buttons that call the app's commands, the same way on every screen. */
export function useAction(what: string): Action {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(
    async (key: string, action: () => Promise<unknown>) => {
      setBusy(key);
      setError(null);
      try {
        await action();
        return true;
      } catch (err) {
        setError(errorText(err));
        logError(`${what} failed`, err);
        return false;
      } finally {
        setBusy(null);
      }
    },
    [what],
  );

  return { busy, error, run };
}
