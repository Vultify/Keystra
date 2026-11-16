// @vitest-environment happy-dom
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LIGHT_COUNT } from "@/lib/bindings/constants";
import type { Pattern } from "@/lib/patterns";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@/lib/logging", () => ({ logError: vi.fn() }));

import { usePatternEditor } from "./use-pattern-editor";

const W = 14;
const A = 9;
const dark = (): Pattern => Array<string>(LIGHT_COUNT).fill("#000000");
const painted = (light: number, color: string): Pattern => dark().map((c, i) => (i === light ? color : c));

/** Starts the editor on pattern 1 with the keyboard holding `keyboard`, and holds the next save open. */
async function openEditor(keyboard: Pattern) {
  let finishSave!: (saved: Pattern) => void;
  invoke.mockImplementation((command: string) => {
    if (command === "read_pattern") return Promise.resolve({ keyboard, savedCopy: null });
    if (command === "save_pattern") return new Promise((resolve) => (finishSave = resolve));
    return Promise.reject(new Error(`unexpected ${command}`));
  });
  const hook = renderHook(() => usePatternEditor(0, true));
  await waitFor(() => expect(hook.result.current.pattern).not.toBeNull());
  return { ...hook, finishSave: (saved: Pattern) => finishSave(saved) };
}

function paintKey(result: { current: ReturnType<typeof usePatternEditor> }, light: number, color: string) {
  act(() => result.current.clearSelection());
  act(() => result.current.toggle(light));
  act(() => result.current.paint(color));
}

describe("usePatternEditor", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("doesn't count painting a key its own color as an edit", async () => {
    const { result } = await openEditor(dark());
    paintKey(result, W, "#000000");
    expect(result.current.dirty).toBe(false);
  });

  it("keeps keys painted while a save is running", async () => {
    const { result, finishSave } = await openEditor(dark());
    paintKey(result, W, "#FF0000");
    act(() => result.current.save());
    paintKey(result, A, "#00FF00");

    await act(async () => finishSave(painted(W, "#FF0000")));

    expect(result.current.dirty).toBe(true);
    expect(result.current.pattern?.[W]).toBe("#FF0000");
    expect(result.current.pattern?.[A]).toBe("#00FF00");
  });

  it("keeps an undo made while a save is running", async () => {
    const { result, finishSave } = await openEditor(dark());
    paintKey(result, W, "#FF0000");
    act(() => result.current.save());
    paintKey(result, W, "#000000");

    await act(async () => finishSave(painted(W, "#FF0000")));

    expect(result.current.dirty).toBe(true);
    expect(result.current.pattern?.[W]).toBe("#000000");
  });

  it("forgets the edits once the save that sent them lands", async () => {
    const { result, finishSave } = await openEditor(dark());
    paintKey(result, W, "#FF0000");
    act(() => result.current.save());

    await act(async () => finishSave(painted(W, "#FF0000")));

    expect(result.current.dirty).toBe(false);
    expect(result.current.pattern?.[W]).toBe("#FF0000");
  });
});
