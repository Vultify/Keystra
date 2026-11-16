import { describe, expect, it } from "vitest";

import { effectPreview } from "./effect-motion";
import { KEYS, type Key } from "./keyboard-layout";
import { OFFERED_LIGHTING_MODES } from "./lighting";
import type { LightingSettings } from "./profiles";

const lighting = (mode: number, direction = 0, rainbow = false): LightingSettings => ({
  mode,
  brightness: 4,
  speed: 2,
  direction,
  rainbow,
  color: "#FF0000",
});

const key = (label: string) => KEYS.find((k) => k.label === label || k.sub === label)!;
const rowOf = (k: Key) => (k.y === 0 ? 0 : Math.round(k.y - 0.25));
/** Keys brighter than half, at one moment. */
const litAt = (mode: number, seconds: number, direction = 0) =>
  KEYS.filter((k) => effectPreview(lighting(mode, direction)).at(k, seconds).level > 0.5);

describe("effect previews", () => {
  it("keeps every level between 0 and 1 for every offered effect and option", () => {
    for (const [id, mode] of OFFERED_LIGHTING_MODES) {
      for (let direction = 0; direction < Math.max(1, mode.options.length); direction++) {
        const preview = effectPreview(lighting(id, direction, true));
        for (const t of [0, 0.37, 1.9, 7.3]) {
          for (const k of KEYS) {
            const { level } = preview.at(k, t);
            expect(level).toBeGreaterThanOrEqual(0);
            expect(level).toBeLessThanOrEqual(1);
          }
        }
      }
    }
  });

  it("keeps still effects still", () => {
    for (const mode of [0, 1, 13]) expect(effectPreview(lighting(mode)).moving).toBe(false);
  });

  it("sweeps Flowing Light the way its direction says", () => {
    const right = effectPreview(lighting(4, 0));
    expect(right.at(key("ESC"), 0).level).toBeGreaterThan(right.at(key("PAUSE"), 0).level);
  });

  it("lights only one row at a time for Meteor", () => {
    for (const t of [0.5, 3, 8]) {
      const rows = new Set(litAt(18, t).map(rowOf));
      expect(rows.size).toBeLessThanOrEqual(1);
    }
  });

  it("brings Layers in from opposite corners, meets in the middle, then goes dark", () => {
    // Speed 2 makes a 6.5 second loop. Early on the blocks are near the corners, not the middle.
    const early = litAt(9, 0.1);
    expect(early.some((k) => k.x < 3 && k.y < 3)).toBe(true);
    expect(early.some((k) => k.x > 12 && k.y > 3)).toBe(true);
    expect(early.some((k) => k.x > 6 && k.x < 10)).toBe(false);
    // Mid loop the ring spins over the middle of the letter keys.
    expect(litAt(9, 2).every((k) => k.x > 4 && k.x < 11)).toBe(true);
    // Late in the loop the board is dark.
    expect(litAt(9, 5.5)).toHaveLength(0);
  });

  it("draws Sine Wave as a thin line: at most two rows lit in any column", () => {
    const lit = litAt(10, 1.3);
    const perColumn = new Map<number, number>();
    for (const k of lit) perColumn.set(Math.round(k.x), (perColumn.get(Math.round(k.x)) ?? 0) + 1);
    for (const count of perColumn.values()) expect(count).toBeLessThanOrEqual(2);
  });

  it("keeps the whole board lit and turning for a rainbow Loop", () => {
    const loop = effectPreview(lighting(15, 0, true));
    expect(KEYS.every((k) => loop.at(k, 1).level === 1)).toBe(true);
    expect(loop.at(key("ESC"), 0).hue).not.toBe(loop.at(key("ESC"), 1).hue);
  });

  it("lights only the pressed key's row for Radiant", () => {
    const rows = new Set(litAt(14, 0.8).map(rowOf));
    expect([...rows]).toEqual([rowOf(key("J"))]);
  });

  it("acts out key presses only on the demo keys", () => {
    const shadow = effectPreview(lighting(8));
    expect(shadow.at(key("W"), 0).level).toBe(1);
    expect(shadow.at(key("Q"), 0).level).toBe(0);
    const silentSnow = effectPreview(lighting(19));
    expect(silentSnow.at(key("W"), 0).level).toBe(0);
    expect(silentSnow.at(key("Q"), 0).level).toBe(1);
  });
});
