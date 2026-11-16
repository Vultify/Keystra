import { describe, expect, it } from "vitest";

import { LIGHT_COUNT } from "./bindings/constants";
import { KEYS, KEY_LIGHTS } from "./keyboard-layout";

describe("keyboard layout", () => {
  it("has the MX 8.2's 87 keys", () => {
    expect(KEYS).toHaveLength(87);
  });

  it("gives every key its own light", () => {
    expect(new Set(KEY_LIGHTS).size).toBe(KEYS.length);
    for (const light of KEY_LIGHTS) {
      expect(light).toBeGreaterThanOrEqual(0);
      expect(light).toBeLessThan(LIGHT_COUNT);
    }
  });

  it("numbers lights by column and row like the keyboard", () => {
    const lightOf = (label: string) => KEYS.find((key) => key.label === label)?.light;
    // Light = column x 6 + row; W is column 2 of the Tab row, the arrow up column 15 of the Shift row.
    expect(lightOf("W")).toBe(14);
    expect(lightOf("A")).toBe(9);
    expect(lightOf("△")).toBe(94);
    expect(lightOf("PAUSE")).toBe(96);
  });
});
