// Looping examples of each lighting effect for the keyboard picture: how bright each key is, and its
// hue when that moves, at any moment. Matched by eye to phone videos of the MX 8.2 where we have them
// (Layers, Sine Wave, Loop, Snowfall, Meteor, Radiant); the rest follow MagCrate's internal effect names.

import { MAX_LEVEL } from "./bindings/constants";
import { effectGuide } from "./effect-guide";
import { BOARD_HEIGHT, BOARD_WIDTH, KEYS, type Key } from "./keyboard-layout";
import { lightingMode } from "./lighting";
import type { LightingSettings } from "./profiles";

/** One key at one moment: brightness from 0 (dark) to 1, and a hue in degrees when the effect sets its own. */
export type KeyLook = { level: number; hue: number | null };

export type EffectPreview = {
  /** Whether the picture changes over time. */
  moving: boolean;
  /** How a key looks `seconds` after the preview started. */
  at: (key: Key, seconds: number) => KeyLook;
};

const centerX = (key: Key) => (key.x + key.w / 2) / BOARD_WIDTH;
const centerY = (key: Key) => (key.y + 0.5) / BOARD_HEIGHT;
/** Row 0 is the function row, 5 the bottom row. */
const rowOf = (key: Key) => (key.y === 0 ? 0 : Math.round(key.y - 0.25));
const frac = (value: number) => value - Math.floor(value);
const clamp = (value: number) => Math.min(1, Math.max(0, value));

/** A steady pseudo-random number from 0 to 1, so random-looking effects repeat the same way. */
const scatter = (seed: number) => frac(Math.sin(seed * 12.9898) * 43758.5453);

/** Distance from the board's center, with the board's long side counted as 1. */
const fromCenter = (key: Key) => Math.hypot(centerX(key) - 0.5, (centerY(key) - 0.5) * (BOARD_HEIGHT / BOARD_WIDTH));

/** Angle around the board's center as a fraction of a turn (0 to 1), clockwise on screen. */
const angleOf = (key: Key) => frac(Math.atan2(centerY(key) - 0.5, centerX(key) - 0.5) / (Math.PI * 2));

/** How lit a key is behind a moving head: 1 right at it, fading to 0 `length` behind it. */
const trail = (behind: number, length: number) => (behind >= 0 && behind < length ? 1 - behind / length : 0);

/** Distance around a loop of length 1, so 0.95 and 0.05 are 0.1 apart. */
const loopGap = (a: number, b: number) => {
  const gap = Math.abs(frac(a) - frac(b));
  return Math.min(gap, 1 - gap);
};

function keyAt(label: string): Key {
  // Number keys carry their digit as the printed legend under the symbol.
  const key = KEYS.find((k) => k.label === label || k.sub === label);
  if (!key) throw new Error(`no key labeled ${label}`);
  return key;
}

/** Keys used to act out typing for the effects that react to key presses, one after another. */
const DEMO_PRESSES = ["W", "H", "L", "5"].map(keyAt);

/** The order Meteor streaks run through the rows, as filmed. */
const METEOR_ROWS = [2, 3, 5, 0, 4, 1];

/** Layers' meeting point, over U and J in the middle of the letter keys, in key units. */
const MEET = { x: 7.7, y: 2.9 };
/** Where Layers' two blocks start: off the top-left and bottom-right corners. */
const LAYER_ENDS = [
  { x: 0.8, y: 1.2 },
  { x: 14.5, y: 4.4 },
];

/**
 * Layers, as filmed: two blocks close in from opposite corners, meet over U and J as a ring that
 * spins while it shrinks and grows, then the ring spreads out past every edge, then a dark pause.
 * `phase` runs 0 to 1.
 */
function converge(key: Key, phase: number): number {
  const kx = key.x + key.w / 2;
  const ky = key.y + 0.5;
  const block = (at: { x: number; y: number }) => 1 - Math.hypot(kx - at.x, ky - at.y) / 2.6;
  const travel = (s: number) => Math.max(...LAYER_ENDS.map((end) => block({ x: end.x + (MEET.x - end.x) * s, y: end.y + (MEET.y - end.y) * s })));
  const distance = Math.hypot(kx - MEET.x, ky - MEET.y);
  const ring = (radius: number) => 1 - Math.abs(distance - radius) / 1.1;
  if (phase < 0.15) return travel(phase / 0.15);
  if (phase < 0.5) {
    const spin = (phase - 0.15) / 0.35;
    const arcs = 0.5 + 0.5 * Math.cos(2 * (Math.atan2(ky - MEET.y, kx - MEET.x) - spin * Math.PI * 4));
    return ring(1.8 - 1.1 * Math.sin(Math.PI * spin)) * (0.55 + 0.45 * arcs);
  }
  // The ring grows until it has passed the farthest key.
  if (phase < 0.68) return ring(1.8 + ((phase - 0.5) / 0.18) * 9.5);
  return 0;
}

/** Seconds per loop: about 5 at the slowest speed, 1.5 at the fastest. */
function loopSeconds(lighting: MotionSettings): number {
  const speed = lightingMode(lighting.mode)?.speed ? lighting.speed : MAX_LEVEL / 2;
  return 5 - (speed / MAX_LEVEL) * 3.5;
}

/** Where a key sits along a direction of travel, 0 at the start and 1 at the end. */
function along(direction: string, key: Key): number {
  switch (direction) {
    case "Left":
      return 1 - centerX(key);
    case "Down":
      return centerY(key);
    case "Up":
      return 1 - centerY(key);
    default:
      return centerX(key);
  }
}

/** The lighting settings that change how an effect moves; color and brightness don't. */
type MotionSettings = Pick<LightingSettings, "mode" | "direction" | "speed" | "rainbow">;

const lit = (level: number, hue: number | null = null): KeyLook => ({ level: clamp(level), hue });
const STILL: EffectPreview = { moving: false, at: () => lit(1) };

export function effectPreview(lighting: MotionSettings): EffectPreview {
  const mode = lightingMode(lighting.mode);
  const direction = mode?.options[lighting.direction] ?? "";
  const seconds = loopSeconds(lighting);
  const moving = (look: (key: Key, u: number) => KeyLook, loop = seconds): EffectPreview => ({
    moving: true,
    at: (key, t) => look(key, t / loop),
  });

  switch (effectGuide(lighting.mode).preview) {
    case "off":
      return { moving: false, at: () => lit(0) };
    case "still":
    case "pattern":
      return STILL;
    case "outside":
      // Screen and music effects can't be shown, so they get a slow, gentle shimmer.
      return moving((key, u) => lit(0.6 + 0.4 * Math.sin(Math.PI * 2 * (u + scatter(key.light)))), 6);
    case "cycle":
      return moving((_, u) => lit(1, frac(u) * 360), 14 - (lighting.speed / MAX_LEVEL) * 10);
    case "breathe":
      return moving((_, u) => lit(0.12 + 0.88 * (0.5 - 0.5 * Math.cos(Math.PI * 2 * u))), seconds * 1.5);
    case "sweep":
      return moving((key, u) => lit(0.12 + 0.88 * trail(loopGap(along(direction, key) * 0.7, u), 0.2)));
    case "neon":
      return moving((key, u) => lit(trail(loopGap(along(direction, key), u), 0.08)));
    case "layers":
      return moving((key, u) => lit(converge(key, frac(u))), seconds * 2);
    case "wave":
      // A thin wavy line across the whole board, rolling sideways.
      return moving((key, u) => {
        const crest = 2.5 + 2.3 * Math.sin(Math.PI * 2 * (1.2 * centerX(key) - u));
        return lit(1 - Math.abs(rowOf(key) - crest) / 0.9);
      });
    case "snake": {
      // A short trail along a path: each row in turn for Zigzag, or round and inward for Spiral.
      const path =
        direction === "Spiral"
          ? (key: Key) => frac(angleOf(key) + fromCenter(key) * 1.5)
          : (key: Key) => (rowOf(key) + (rowOf(key) % 2 === 0 ? centerX(key) : 1 - centerX(key))) / 6;
      return moving((key, u) => lit(trail(frac(u - path(key)), 0.12)), seconds * 2);
    }
    case "rings": {
      // Kaleidoscope: color rings spreading out (or in) from the center, the whole board lit.
      const way = direction === "Inward" ? 1 : -1;
      return moving((key, u) => {
        const ring = frac(fromCenter(key) * 2 + way * u);
        return lit(0.35 + 0.65 * (0.5 + 0.5 * Math.cos(Math.PI * 2 * ring)), lighting.rainbow ? ring * 360 : null);
      });
    }
    case "loop": {
      // The whole board lit, with the colors turning around it like a color wheel.
      const way = direction === "Clockwise" ? -1 : 1;
      return moving((key, u) => {
        const turn = frac(angleOf(key) + way * u);
        return lighting.rainbow
          ? lit(1, turn * 360)
          : lit(0.35 + 0.65 * (0.5 + 0.5 * Math.cos(Math.PI * 2 * turn)));
      }, seconds * 1.5);
    }
    case "twinkle":
      // Star Dots: random keys light up and fade, a few at a time.
      return moving((key, u) => lit(trail(frac(u + scatter(key.light)), 0.18)), seconds * 1.5);
    case "dazzle":
      // Color Grid: every key lit, each jumping to a new random color now and then.
      return moving((key, u) => {
        const step = Math.floor(u * 3 + scatter(key.light) * 3);
        return lit(0.55 + 0.45 * scatter(key.light * 7 + step), scatter(key.light + step * 13) * 360);
      });
    case "fall":
      // Snowfall: short drops fall down each column, each column at its own pace.
      return moving((key, u) => {
        const column = Math.round(centerX(key) * 18);
        const head = frac(u * (0.8 + 0.6 * scatter(column)) + scatter(column + 50)) * 1.5 - 0.25;
        return lit(trail(head - centerY(key), 0.32));
      }, seconds * 1.2);
    case "meteor":
      // One streak with a tail races left to right along a row, then the next row.
      return moving((key, u) => {
        const step = frac(u) * METEOR_ROWS.length;
        if (rowOf(key) !== METEOR_ROWS[Math.floor(step)]) return lit(0);
        const head = frac(step) * 1.35 - 0.05;
        return lit(trail(head - centerX(key), 0.3));
      }, seconds * 3);
    case "ripple": {
      // Water Ripple: a ring spreads from a pressed key and fades as it goes.
      const press = keyAt("G");
      return moving((key, u) => {
        const radius = frac(u) * 0.9;
        const distance = Math.hypot(centerX(key) - centerX(press), (centerY(key) - centerY(press)) * (BOARD_HEIGHT / BOARD_WIDTH));
        return lit((1 - Math.abs(distance - radius) / 0.07) * (1 - frac(u)));
      });
    }
    case "radiant": {
      // Light shoots out sideways along the pressed key's row; the rest stays dark.
      const press = keyAt("J");
      return moving((key, u) => {
        if (rowOf(key) !== rowOf(press)) return lit(0);
        const reach = frac(u) * 1.1;
        return lit(trail(reach - Math.abs(centerX(key) - centerX(press)), 0.25));
      });
    }
    case "press":
    case "press-off": {
      // Demo key presses, one after another: they light up (or go dark, for press-off).
      const lights = effectGuide(lighting.mode).preview === "press";
      return moving((key, u) => {
        const turn = DEMO_PRESSES.findIndex((demo) => demo.light === key.light);
        if (turn < 0) return lit(lights ? 0 : 1);
        const since = frac(u - turn / DEMO_PRESSES.length);
        return lights ? lit(trail(since, 0.25)) : lit(since < 0.3 ? since / 0.3 : 1);
      }, seconds * 1.5);
    }
  }
}
