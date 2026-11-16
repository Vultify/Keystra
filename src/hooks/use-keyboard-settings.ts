import { useCallback, useEffect, useState } from "react";

import { useAction } from "@/hooks/use-action";
import { onKeyboardSettings, readKeyboard, type KeyboardStatus, type Snapshot } from "@/lib/keyboard";

export type KeyboardSettings = {
  /** What the keyboard holds, or `null` while disconnected or before the first read. */
  snapshot: Snapshot | null;
  reading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
};

/**
 * The keyboard's current settings for the Diagnostics page: read when it opens and each time the
 * keyboard connects, then kept current by the event Keystra sends whenever it changes them.
 */
export function useKeyboardSettings(status: KeyboardStatus | null): KeyboardSettings {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const { busy, error, run } = useAction("reading the keyboard");
  const deviceId = status?.state === "connected" ? status.info.deviceId : null;

  const refresh = useCallback(async () => {
    await run("read", async () => setSnapshot(await readKeyboard()));
  }, [run]);

  useEffect(() => {
    const unlisten = onKeyboardSettings(setSnapshot);
    return () => {
      unlisten.then((stop) => stop()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    if (deviceId === null) {
      setSnapshot(null);
      return;
    }
    void refresh();
  }, [deviceId, refresh]);

  return { snapshot, reading: busy !== null, error, refresh };
}
