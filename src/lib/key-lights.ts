import { effectGuide } from "./effect-guide";
import type { EffectPreview, KeyLook } from "./effect-motion";
import { BOARD_WIDTH, KEYS, type Key } from "./keyboard-layout";
import type { Pattern } from "./patterns";
import type { LightingSettings } from "./profiles";

const DARK = "#000000";

/** The color under one key before the effect moves it, or `null` when the key stays dark. */
export function baseColor(
  lighting: Pick<LightingSettings, "mode" | "rainbow" | "color">,
  pattern: Pattern | null,
): (key: Key) => string | null {
  const preview = effectGuide(lighting.mode).preview;
  if (preview === "off") return () => null;
  if (preview === "pattern") {
    // The keyboard's saved pattern, key by key. Black keys stay dark.
    return (key) => {
      const color = pattern?.[key.light];
      return color && color.toUpperCase() !== DARK ? color : null;
    };
  }
  const hueAt = (key: Key) => Math.round(((key.x + key.w / 2) / BOARD_WIDTH) * 330);
  if (preview === "outside") return (key) => `hsl(${hueAt(key)} 45% 60%)`;
  return lighting.rainbow ? (key) => `hsl(${hueAt(key)} 90% 60%)` : () => lighting.color;
}

/** How one key's light is drawn at one moment: the effect's own hue when it sets one, else its base color. */
export function lightOf(base: string | null, look: KeyLook): { fill: string; opacity: number } {
  if (base === null) return { fill: "transparent", opacity: 0 };
  return { fill: look.hue === null ? base : `hsl(${Math.round(look.hue)} 90% 60%)`, opacity: look.level };
}

/**
 * The moment of a moving effect that shows it best when it can't play: the frame with the most
 * contrast between keys, leaning towards brighter frames. Still effects use their only frame.
 */
export function restFrame(preview: EffectPreview): number {
  if (!preview.moving) return 0;
  let best = 0;
  let bestScore = -1;
  for (let seconds = 0; seconds <= 6; seconds += 0.25) {
    const levels = KEYS.map((key) => preview.at(key, seconds).level);
    const mean = levels.reduce((sum, level) => sum + level, 0) / levels.length;
    const spread = Math.sqrt(levels.reduce((sum, level) => sum + (level - mean) ** 2, 0) / levels.length);
    const score = mean + 2 * spread;
    if (score > bestScore) {
      best = seconds;
      bestScore = score;
    }
  }
  return best;
}
