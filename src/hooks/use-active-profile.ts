import { useCallback, useEffect, useRef, useState } from "react";

import { errorText } from "@/lib/errors";
import { logError } from "@/lib/logging";
import { saveProfile, type Profile, type ProfileList } from "@/lib/profiles";

export type ActiveProfile = {
  /** The active profile as the screens should show it, including changes still on their way. */
  profile: Profile | null;
  /** Shows a change on screen without saving it, for a slider being dragged. */
  preview: (next: Profile) => void;
  /** Saves a change and sends it to the keyboard. */
  commit: (next: Profile) => void;
  /** Shows a change at once and saves it after `wait` ms without another one, for controls that report every step. */
  commitLater: (next: Profile, wait: number) => void;
  saving: boolean;
  problem: string | null;
};

type Waiting = { next: Profile; timer: number };

/** The profile the Lighting and Performance pages edit: changes save and reach the keyboard live. */
export function useActiveProfile(list: ProfileList | null): ActiveProfile {
  const stored = list?.profiles.find((profile) => profile.id === list.activeId) ?? null;
  const storedKey = stored ? JSON.stringify(stored) : null;
  const [draft, setDraft] = useState<Profile | null>(stored);
  const [pending, setPending] = useState(0);
  const [problem, setProblem] = useState<string | null>(null);
  // Only the newest change may report a problem; older ones may have been merged into it.
  const newest = useRef(0);
  const dragging = useRef(false);
  const waiting = useRef<Waiting | null>(null);

  const save = useCallback((next: Profile) => {
    const number = ++newest.current;
    setProblem(null);
    setPending((count) => count + 1);
    saveProfile(next)
      .then(({ applyError }) => {
        if (number === newest.current && applyError) setProblem(`Saved, but the keyboard didn't take it: ${applyError}`);
      })
      .catch((err) => {
        if (number === newest.current) setProblem(errorText(err));
        logError("saving the active profile failed", err);
      })
      .finally(() => setPending((count) => count - 1));
  }, []);

  /** Sends a change that's still waiting, rather than dropping it. */
  const flush = useCallback(() => {
    const held = waiting.current;
    if (!held) return;
    window.clearTimeout(held.timer);
    waiting.current = null;
    save(held.next);
  }, [save]);

  // A different profile became active (from Profiles, or later the tray or auto-switch): start
  // fresh on it, and let no old result show up under its name.
  const storedId = stored?.id ?? null;
  useEffect(() => {
    flush();
    dragging.current = false;
    newest.current++;
    setProblem(null);
    setDraft(storedKey ? (JSON.parse(storedKey) as Profile) : null);
  }, [storedId, flush]);

  // Follow the saved profile, unless one of our own changes is still being made or saved.
  useEffect(() => {
    if (pending === 0 && !dragging.current && !waiting.current) {
      setDraft(storedKey ? (JSON.parse(storedKey) as Profile) : null);
    }
  }, [storedKey, pending]);

  // A change still waiting when the page closes is sent, not lost.
  useEffect(() => flush, [flush]);

  const commit = useCallback(
    (next: Profile) => {
      if (waiting.current) window.clearTimeout(waiting.current.timer);
      waiting.current = null;
      dragging.current = false;
      setDraft(next);
      save(next);
    },
    [save],
  );

  const commitLater = useCallback(
    (next: Profile, wait: number) => {
      if (waiting.current) window.clearTimeout(waiting.current.timer);
      setDraft(next);
      waiting.current = { next, timer: window.setTimeout(flush, wait) };
    },
    [flush],
  );

  const preview = useCallback((next: Profile) => {
    dragging.current = true;
    setDraft(next);
  }, []);

  return { profile: draft, preview, commit, commitLater, saving: pending > 0, problem };
}
