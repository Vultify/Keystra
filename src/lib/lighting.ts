import { LIGHTING_MODES, type LightingMode } from "./bindings/constants";

export type { LightingMode } from "./bindings/constants";

/** MagCrate's name for a lighting effect, with one fallback for effects Keystra doesn't know. */
function lightingModeName(mode: number): string {
  return LIGHTING_MODES[mode]?.name ?? `Unknown effect (mode ${mode})`;
}

export function lightingMode(mode: number): LightingMode | undefined {
  return LIGHTING_MODES[mode];
}

/** The effects the MX 8.2 offers, by ID. */
export const OFFERED_LIGHTING_MODES: [number, LightingMode][] = Object.entries(LIGHTING_MODES)
  .filter(([, mode]) => mode.offered)
  .map(([id, mode]) => [Number(id), mode]);

export type LightingDescription = {
  effect: string;
  /** "Rainbow", a "#RRGGBB" color, or `null` for effects that pick their own colors. */
  color: string | null;
  /** The effect's option, like Direction: Left, or `null` when it has none. */
  option: { label: string; value: string } | null;
};

/** One way to describe lighting for every screen, following each effect's rules from the Rust table. */
export function describeLighting(mode: number, rainbow: boolean, color: string | null, option: number): LightingDescription {
  const info = LIGHTING_MODES[mode];
  return {
    effect: lightingModeName(mode),
    color: info?.usesColor ? (rainbow ? "Rainbow" : color) : null,
    option: info?.optionLabel
      ? { label: info.optionLabel, value: info.options[option] ?? `Option ${option + 1}` }
      : null,
  };
}

/** Quick color choices for the lighting color and for painting patterns. */
export const COLOR_SWATCHES = [
  { name: "White", color: "#FFFFFF" },
  { name: "Red", color: "#FF0000" },
  { name: "Orange", color: "#FF8000" },
  { name: "Yellow", color: "#FFFF00" },
  { name: "Green", color: "#00FF00" },
  { name: "Cyan", color: "#00FFFF" },
  { name: "Blue", color: "#0000FF" },
  { name: "Purple", color: "#7A30F8" },
  { name: "Pink", color: "#FF00FF" },
];
