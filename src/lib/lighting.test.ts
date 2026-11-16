import { describe, expect, it } from "vitest";

import { LIGHTING_MODES } from "./bindings/constants";
import { effectGuide } from "./effect-guide";
import { carriedOption, describeLighting } from "./lighting";

const FLOWING_LIGHT = 4;
const SPECTRUM_CYCLE = 3;
const CUSTOM = 13;

describe("describeLighting", () => {
  it("shows the color or rainbow only for effects that use one", () => {
    expect(describeLighting(FLOWING_LIGHT, false, "#FF0000", 0).color).toBe("#FF0000");
    expect(describeLighting(FLOWING_LIGHT, true, "#FF0000", 0).color).toBe("Rainbow");
    expect(describeLighting(SPECTRUM_CYCLE, true, "#FF0000", 0).color).toBeNull();
  });

  it("names the effect's option the way the table does", () => {
    expect(describeLighting(FLOWING_LIGHT, false, "#FF0000", 1).option).toEqual({ label: "Direction", value: "Left" });
    expect(describeLighting(CUSTOM, false, "#FF0000", 2).option).toEqual({ label: "Pattern", value: "Pattern 3" });
    expect(describeLighting(SPECTRUM_CYCLE, false, "#FF0000", 0).option).toBeNull();
  });

  it("falls back for an effect Keystra doesn't know", () => {
    expect(describeLighting(99, false, null, 0).effect).toBe("Unknown effect (mode 99)");
  });
});

const NEON = 12;
const LOOP = 15;

describe("carriedOption", () => {
  it("keeps an option only when the new effect has one with the same name", () => {
    expect(carriedOption(FLOWING_LIGHT, 1, NEON)).toBe(1); // Left stays Left
    expect(carriedOption(NEON, 1, LOOP)).toBe(0); // Left means nothing to Loop
    expect(carriedOption(FLOWING_LIGHT, 3, CUSTOM)).toBe(0); // Up never becomes Pattern 4
    expect(carriedOption(SPECTRUM_CYCLE, 0, FLOWING_LIGHT)).toBe(0);
  });
});

describe("option names the screens depend on", () => {
  const allOptions = new Set(Object.values(LIGHTING_MODES).flatMap((mode) => mode.options));

  it("still exist in the effect table", () => {
    // effect-motion.ts and lighting-adjust.tsx pick motion and icons by these names.
    for (const name of ["Right", "Left", "Up", "Down", "Zigzag", "Spiral", "Outward", "Inward", "Clockwise", "Counterclockwise"]) {
      expect(allOptions.has(name), name).toBe(true);
    }
  });

  it("match the effects whose picture shows a saved pattern", () => {
    for (const [id, mode] of Object.entries(LIGHTING_MODES)) {
      expect(mode.optionLabel === "Pattern", mode.name).toBe(effectGuide(Number(id)).preview === "pattern");
    }
  });
});
