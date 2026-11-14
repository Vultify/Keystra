import { expect, it } from "vitest";

import { errorText } from "./errors";

it("gives the message without an 'Error:' prefix", () => {
  expect(errorText(new Error("keyboard is busy"))).toBe("keyboard is busy");
  expect(errorText("profile not found")).toBe("profile not found");
});
