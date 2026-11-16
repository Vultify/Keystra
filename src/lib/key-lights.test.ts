import { describe, expect, it } from "vitest";

import { effectPreview } from "./effect-motion";
import { baseColor, lightOf, restFrame } from "./key-lights";
import { KEYS } from "./keyboard-layout";

const settings = (mode: number) => ({ mode, direction: 0, speed: 2, rainbow: false });

describe("key lights", () => {
  it("rests a still effect on its only frame", () => {
    expect(restFrame(effectPreview(settings(1)))).toBe(0);
  });

  it("rests a moving effect on a frame where something is lit", () => {
    for (const mode of [2, 4, 6, 9, 11, 17, 18]) {
      const preview = effectPreview(settings(mode));
      const at = restFrame(preview);
      expect(KEYS.some((key) => preview.at(key, at).level > 0.5), `mode ${mode}`).toBe(true);
    }
  });

  it("leaves every key dark when the effect is off", () => {
    const base = baseColor({ mode: 0, rainbow: false, color: "#FF0000" }, null);
    expect(KEYS.every((key) => base(key) === null)).toBe(true);
    expect(lightOf(null, { level: 1, hue: null })).toEqual({ fill: "transparent", opacity: 0 });
  });
});
