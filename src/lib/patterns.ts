import { invoke } from "@tauri-apps/api/core";

import type { Pattern } from "./bindings/Pattern";
import type { PatternReading } from "./bindings/PatternReading";

export type { Pattern } from "./bindings/Pattern";
export type { PatternReading } from "./bindings/PatternReading";

/** Reads one of the keyboard's 5 saved patterns (slot 0 is Pattern 1). */
export function readPattern(slot: number): Promise<PatternReading> {
  return invoke<PatternReading>("read_pattern", { slot });
}

/** Saves a pattern to the keyboard: one write to its memory, confirmed by reading it back. */
export function savePattern(slot: number, pattern: Pattern): Promise<Pattern> {
  return invoke<Pattern>("save_pattern", { slot, pattern });
}

/** A copy of `pattern` with the given lights set to `color`. */
export function paintLights(pattern: Pattern, lights: ReadonlySet<number>, color: string): Pattern {
  return pattern.map((current, light) => (lights.has(light) ? color.toUpperCase() : current));
}

/** Makes the keyboard's current pattern Keystra's copy, for when it was changed elsewhere on purpose. */
export function keepKeyboardPattern(slot: number): Promise<PatternReading> {
  return invoke<PatternReading>("keep_keyboard_pattern", { slot });
}

/** Whether two patterns hold the same colors, ignoring letter case. */
export function samePattern(a: Pattern, b: Pattern): boolean {
  return a.length === b.length && a.every((color, light) => color.toUpperCase() === b[light].toUpperCase());
}
