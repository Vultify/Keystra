import { LIGHTING_MODES } from "./bindings/constants";

/** MagCrate's name for a lighting effect, with one fallback for effects Keystra doesn't know. */
export function lightingModeName(mode: number): string {
  return LIGHTING_MODES[mode]?.name ?? `Unknown effect (mode ${mode})`;
}

/** False for effects that pick their own colors, where a chosen color has no effect. */
export function lightingUsesColor(mode: number): boolean {
  return LIGHTING_MODES[mode]?.usesColor ?? false;
}
