// @vitest-environment happy-dom
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Profile, ProfileList } from "@/lib/profiles";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@/lib/logging", () => ({ logError: vi.fn() }));

import { useActiveProfile } from "./use-active-profile";

function profile(id: number, color = "#FF0000"): Profile {
  return {
    id,
    name: `Profile ${id}`,
    onboardSlot: 0,
    lighting: { mode: 1, brightness: 4, speed: 2, direction: 0, rainbow: false, color },
    pollingRateHz: 2000,
    linkedApps: [],
  };
}

const listOf = (activeId: number, ...profiles: Profile[]): ProfileList => ({ profiles, activeId, readOnly: null, unsentId: null });

/** A promise the test settles by hand, to control the order saves finish in. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

describe("useActiveProfile", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("lets only the newest change report a problem", async () => {
    const older = deferred<unknown>();
    const newer = deferred<unknown>();
    invoke.mockReturnValueOnce(older.promise).mockReturnValueOnce(newer.promise);
    const { result } = renderHook(() => useActiveProfile(listOf(1, profile(1))));

    act(() => result.current.commit(profile(1, "#00FF00")));
    act(() => result.current.commit(profile(1, "#0000FF")));
    await act(async () => {
      older.reject("the keyboard is busy");
      newer.resolve({ profile: profile(1, "#0000FF"), applyError: null });
    });

    expect(result.current.problem).toBeNull();
  });

  it("sends a color that's still settling when the page closes", () => {
    invoke.mockResolvedValue({ profile: profile(1), applyError: null });
    const { result, unmount } = renderHook(() => useActiveProfile(listOf(1, profile(1))));

    act(() => result.current.commitLater(profile(1, "#123456"), 400));
    expect(invoke).not.toHaveBeenCalled();
    unmount();

    expect(invoke).toHaveBeenCalledWith("save_profile", { profile: profile(1, "#123456") });
  });

  it("follows a switch made outside the window, like from the tray", () => {
    const { result, rerender } = renderHook(({ list }) => useActiveProfile(list), {
      initialProps: { list: listOf(1, profile(1), profile(2)) },
    });
    expect(result.current.profile?.id).toBe(1);

    rerender({ list: listOf(2, profile(1), profile(2)) });

    expect(result.current.profile?.id).toBe(2);
  });
});
