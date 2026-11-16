import { useCallback, useEffect, useRef, useState } from "react";

import { useAction } from "@/hooks/use-action";
import { useKeySelection } from "@/hooks/use-key-selection";
import {
  keepKeyboardPattern,
  paintLights,
  readPattern,
  samePattern,
  savePattern,
  type Pattern,
  type PatternReading,
} from "@/lib/patterns";

export type PatternEditor = {
  /** The pattern as the screen shows it: unsaved edits, or what the keyboard holds. `null` until read. */
  pattern: Pattern | null;
  /** Keystra's copy when it differs from the keyboard, for Restore. */
  savedCopy: Pattern | null;
  dirty: boolean;
  selected: ReadonlySet<number>;
  /** "read", "save" or "keep" while one is running. */
  busy: string | null;
  error: string | null;
  toggle: (light: number) => void;
  selectAll: () => void;
  clearSelection: () => void;
  paint: (color: string) => void;
  restoreCopy: () => void;
  /** Makes the keyboard's pattern Keystra's copy, without writing to the keyboard. */
  keepKeyboardVersion: () => void;
  discard: () => void;
  save: () => void;
  readAgain: () => void;
};

/**
 * Edits one of the keyboard's saved patterns, keeping unsaved edits per pattern while the page is
 * open. Only Save pattern writes the keyboard, and only when something actually changed.
 */
export function usePatternEditor(slot: number | null, connected: boolean): PatternEditor {
  const { busy, error, run } = useAction("pattern");
  const [readings, setReadings] = useState<Record<number, PatternReading>>({});
  const [drafts, setDrafts] = useState<Record<number, Pattern>>({});
  const { selected, toggle, selectAll, clear } = useKeySelection();
  const [attempt, setAttempt] = useState(0);
  // The pattern a running save is sending, which is what the keyboard will hold once it lands.
  const saving = useRef<Pattern | null>(null);

  // Read the pattern each time it's picked, the keyboard comes back, or Read again is pressed.
  useEffect(() => {
    if (slot === null || !connected) return;
    void run("read", async () => {
      const reading = await readPattern(slot);
      setReadings((current) => ({ ...current, [slot]: reading }));
    });
  }, [slot, connected, run, attempt]);

  const reading = slot === null ? undefined : readings[slot];
  const draft = slot === null ? undefined : drafts[slot];
  const pattern = draft ?? reading?.keyboard ?? null;

  /** Sets this pattern's unsaved edits; edits that match the keyboard aren't edits. */
  const setDraft = useCallback(
    (next: Pattern | null) => {
      if (slot === null) return;
      const keyboard = saving.current ?? readings[slot]?.keyboard;
      const unchanged = next !== null && keyboard !== undefined && samePattern(next, keyboard);
      setDrafts((current) => {
        const { [slot]: _, ...rest } = current;
        return next && !unchanged ? { ...rest, [slot]: next } : rest;
      });
    },
    [slot, readings],
  );

  return {
    pattern,
    savedCopy: reading?.savedCopy ?? null,
    dirty: draft !== undefined,
    selected,
    busy,
    error,
    toggle,
    selectAll,
    clearSelection: clear,
    paint: (color) => {
      if (pattern && selected.size > 0) setDraft(paintLights(pattern, selected, color));
    },
    restoreCopy: () => {
      if (reading?.savedCopy) setDraft(reading.savedCopy);
    },
    keepKeyboardVersion: () => {
      if (slot === null) return;
      void run("keep", async () => {
        const kept = await keepKeyboardPattern(slot);
        setReadings((current) => ({ ...current, [slot]: kept }));
      });
    },
    discard: () => setDraft(null),
    save: () => {
      if (slot === null || !pattern) return;
      const sent = pattern;
      saving.current = sent;
      void run("save", async () => {
        const saved = await savePattern(slot, sent).finally(() => {
          saving.current = null;
        });
        setReadings((current) => ({ ...current, [slot]: { keyboard: saved, savedCopy: null } }));
        // Keep anything painted while the save was running as unsaved edits.
        setDrafts((current) => {
          if (current[slot] === undefined || !samePattern(current[slot], sent)) return current;
          const { [slot]: _, ...rest } = current;
          return rest;
        });
      });
    },
    readAgain: () => setAttempt((count) => count + 1),
  };
}
