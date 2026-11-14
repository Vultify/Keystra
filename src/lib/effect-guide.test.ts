import { describe, expect, it } from "vitest";

import { EFFECT_GROUPS, effectGuide, hasGuide } from "./effect-guide";
import { OFFERED_LIGHTING_MODES } from "./lighting";

describe("effect guide", () => {
  it("describes every effect the keyboard offers", () => {
    const missing = OFFERED_LIGHTING_MODES.filter(([id]) => !hasGuide(id)).map(([, mode]) => mode.name);
    expect(missing).toEqual([]);
  });

  it("puts every effect in a group the Lighting page shows", () => {
    const groups = new Set(EFFECT_GROUPS.map((group) => group.id));
    for (const [id] of OFFERED_LIGHTING_MODES) expect(groups.has(effectGuide(id).group)).toBe(true);
  });

  it("falls back for an effect it doesn't know", () => {
    expect(effectGuide(99).preview).toBe("outside");
  });
});
