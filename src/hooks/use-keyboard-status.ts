import { useLiveValue } from "@/hooks/use-live-value";
import { getKeyboardStatus, onKeyboardStatus, type KeyboardStatus } from "@/lib/keyboard";

/** Live keyboard connection state. `null` until the first answer from the worker. */
export function useKeyboardStatus(): KeyboardStatus | null {
  return useLiveValue(getKeyboardStatus, onKeyboardStatus, "could not read keyboard status");
}
