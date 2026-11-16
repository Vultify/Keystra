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
 * How the keyboard picture shows an effect. The moving kinds are looping examples of the effect's
 * motion (see effect-motion.ts), not a frame-by-frame copy of the keyboard's animation.
 */
type PreviewKind =
  | "off"
  | "still"
  | "pattern"
  | "outside"
  | "cycle"
  | "breathe"
  | "sweep"
  | "layers"
  | "wave"
  | "neon"
  | "snake"
  | "rings"
  | "loop"
  | "twinkle"
  | "dazzle"
  | "fall"
  | "meteor"
  | "ripple"
  | "radiant"
  | "press"
  | "press-off";

/** `hint` is the few words under an effect's tile; `description` is the full sentence. */
export type EffectGuide = { group: EffectGroup; preview: PreviewKind; hint: string; description: string };

const GUIDE: Record<number, EffectGuide> = {
  0: { group: "steady", preview: "off", hint: "All lights off", description: "All lights off." },
  1: { group: "steady", preview: "still", hint: "One steady color", description: "One steady color, or a still rainbow." },
  2: { group: "animated", preview: "breathe", hint: "Fades in and out", description: "The whole board fades in and out." },
  3: { group: "animated", preview: "cycle", hint: "Cycles every color", description: "The whole board slowly cycles through every color." },
  4: { group: "animated", preview: "sweep", hint: "Flows one way", description: "Color flows across the board in one direction." },
  5: { group: "typing", preview: "ripple", hint: "Ripples from keys", description: "Light ripples out from each key you press." },
  6: { group: "animated", preview: "twinkle", hint: "Twinkles at random", description: "Keys twinkle on at random, like stars." },
  7: { group: "animated", preview: "snake", hint: "A snaking trail", description: "A trail of light snakes across the keys." },
  8: { group: "typing", preview: "press", hint: "Lights up on press", description: "Keys light up when you press them, then fade." },
  9: { group: "animated", preview: "layers", hint: "Meets and spins", description: "Layers of light move across the board." },
  10: { group: "animated", preview: "wave", hint: "A rolling wave", description: "A smooth wave of light rolls across the board." },
  11: { group: "animated", preview: "rings", hint: "Rings from the center", description: "Rings of color spread from the center." },
  12: { group: "animated", preview: "neon", hint: "A sweeping line", description: "A line of light sweeps across the board." },
  13: { group: "special", preview: "pattern", hint: "Your own colors", description: "Shows one of the 5 per-key patterns saved on the keyboard." },
  14: { group: "typing", preview: "radiant", hint: "Shoots from keys", description: "Light shoots out from the keys you press." },
  15: { group: "animated", preview: "loop", hint: "Circles the board", description: "Light circles around the board." },
  16: { group: "animated", preview: "dazzle", hint: "Flickering colors", description: "Keys flicker through random colors." },
  17: { group: "animated", preview: "fall", hint: "Falling light", description: "Light falls down the keys like snow." },
  18: { group: "animated", preview: "meteor", hint: "Streaking meteors", description: "Meteors of light streak across the keys." },
  19: { group: "typing", preview: "press-off", hint: "Dims on press", description: "Keys go dark when you press them, then light back up." },
  21: { group: "special", preview: "outside", hint: "Matches your screen", description: "Matches the colors on your screen. Needs MagCrate's helper running." },
  22: { group: "special", preview: "outside", hint: "Moves to sound", description: "Moves to the sound on your PC. Needs MagCrate's helper running." },
};

export function effectGuide(mode: number): EffectGuide {
  return GUIDE[mode] ?? { group: "special", preview: "outside", hint: "Unknown effect", description: "An effect Keystra doesn't know yet." };
}

/** Whether an effect has its own entry, rather than the fallback (for the tests). */
export function hasGuide(mode: number): boolean {
  return mode in GUIDE;
}
