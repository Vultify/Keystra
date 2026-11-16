import { useSyncExternalStore } from "react";

const query = typeof window === "undefined" ? null : window.matchMedia("(prefers-reduced-motion: reduce)");

function subscribe(onChange: () => void) {
  query?.addEventListener("change", onChange);
  return () => query?.removeEventListener("change", onChange);
}

/** Whether the system asks for less motion (Windows' "Animation effects" off), kept current live. */
export function useReducedMotion(): boolean {
  return useSyncExternalStore(subscribe, () => query?.matches ?? false);
}
