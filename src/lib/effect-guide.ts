// Plain descriptions, groups and picture styles for each lighting effect, written from MagCrate's
// internal effect names. Names and settings come from the generated table; this file only adds the words.

export type EffectGroup = "steady" | "animated" | "typing" | "special";

export const EFFECT_GROUPS: { id: EffectGroup; title: string }[] = [
  { id: "steady", title: "Steady" },
  { id: "animated", title: "Animated" },
  { id: "typing", title: "Reacts to typing" },
  { id: "special", title: "Special" },
];

/**
 * How the keyboard picture shows an effect: dark, a still light, light that moves, the whole board
 * cycling colors, a breathing pulse, a saved per-key pattern, or colors that come from outside.
 */
type EffectPreview = "off" | "still" | "moving" | "cycle" | "breathe" | "pattern" | "outside";

export type EffectGuide = { group: EffectGroup; preview: EffectPreview; description: string };

const GUIDE: Record<number, EffectGuide> = {
  0: { group: "steady", preview: "off", description: "All lights off." },
  1: { group: "steady", preview: "still", description: "One steady color, or a still rainbow." },
  2: { group: "animated", preview: "breathe", description: "The whole board fades in and out." },
  3: { group: "animated", preview: "cycle", description: "The whole board slowly cycles through every color." },
  4: { group: "animated", preview: "moving", description: "Color flows across the board in one direction." },
  5: { group: "typing", preview: "moving", description: "Light ripples out from each key you press." },
  6: { group: "animated", preview: "moving", description: "Keys twinkle on at random, like stars." },
  7: { group: "animated", preview: "moving", description: "A trail of light snakes across the keys." },
  8: { group: "typing", preview: "moving", description: "Keys light up when you press them, then fade." },
  9: { group: "animated", preview: "moving", description: "Layers of light move across the board." },
  10: { group: "animated", preview: "moving", description: "A smooth wave of light rolls across the board." },
  11: { group: "animated", preview: "moving", description: "Rings of color spread from the center." },
  12: { group: "animated", preview: "moving", description: "A line of light sweeps across the board." },
  13: { group: "special", preview: "pattern", description: "Shows one of the 5 per-key patterns saved on the keyboard." },
  14: { group: "typing", preview: "moving", description: "Light shoots out from the keys you press." },
  15: { group: "animated", preview: "moving", description: "Light circles around the board." },
  16: { group: "animated", preview: "moving", description: "Keys flicker through random colors." },
  17: { group: "animated", preview: "moving", description: "Light falls down the keys like snow." },
  18: { group: "animated", preview: "moving", description: "Meteors of light streak across the keys." },
  19: { group: "typing", preview: "moving", description: "Keys go dark when you press them, then light back up." },
  21: { group: "special", preview: "outside", description: "Matches the colors on your screen. Needs MagCrate's helper running." },
  22: { group: "special", preview: "outside", description: "Moves to the sound on your PC. Needs MagCrate's helper running." },
};

export function effectGuide(mode: number): EffectGuide {
  return GUIDE[mode] ?? { group: "special", preview: "outside", description: "An effect Keystra doesn't know yet." };
}

/** Whether an effect has its own entry, rather than the fallback (for the tests). */
export function hasGuide(mode: number): boolean {
  return mode in GUIDE;
}
