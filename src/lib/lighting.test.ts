import { describe, expect, it } from "vitest";

import { describeLighting } from "./lighting";

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
