import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { SETTINGS_EVENT, STATUS_EVENT } from "./bindings/constants";
import type { KeyboardStatus } from "./bindings/KeyboardStatus";
import type { Snapshot } from "./bindings/Snapshot";

// Types and constants are generated from the Rust side by `cargo test` (see src/lib/bindings).
export { MAX_LEVEL } from "./bindings/constants";
import { GERMAN_DEVICE_ID } from "./bindings/constants";
export type { DeviceInfo } from "./bindings/DeviceInfo";
export type { Exchange } from "./bindings/Exchange";
export type { KeyboardStatus } from "./bindings/KeyboardStatus";
export type { Lighting } from "./bindings/Lighting";
export type { MagCrateState } from "./bindings/MagCrateState";
export type { Snapshot } from "./bindings/Snapshot";

export function getKeyboardStatus(): Promise<KeyboardStatus> {
  return invoke<KeyboardStatus>("keyboard_status");
}

export function readKeyboard(): Promise<Snapshot> {
  return invoke<Snapshot>("read_keyboard");
}

/** Fires with the keyboard's settings whenever Keystra changes them. */
export function onKeyboardSettings(handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> {
  return listen<Snapshot>(SETTINGS_EVENT, (event) => handler(event.payload));
}

export function onKeyboardStatus(handler: (status: KeyboardStatus) => void): Promise<UnlistenFn> {
  return listen<KeyboardStatus>(STATUS_EVENT, (event) => handler(event.payload));
}

/** The keyboard's own name for an onboard slot: slot 0 is "Profile 1". */
export function onboardSlotLabel(slot: number): string {
  return `Profile ${slot + 1}`;
}

export function formatFirmware(version: number): string {
  return `0x${version.toString(16).toUpperCase().padStart(4, "0")}`;
}

/** Hex dump with trailing zero bytes trimmed, keeping at least the 8-byte header. */
export function toHex(bytes: number[]): string {
  let end = bytes.length;
  while (end > 8 && bytes[end - 1] === 0) end--;
  const shown = bytes.slice(0, end).map((b) => b.toString(16).toUpperCase().padStart(2, "0"));
  return end < bytes.length ? `${shown.join(" ")} …` : shown.join(" ");
}

/** Whether the connected keyboard is the German model, whose two extra keys the picture doesn't show yet. */
export function isGermanBoard(status: KeyboardStatus | null): boolean {
  return status?.state === "connected" && status.info.deviceId === GERMAN_DEVICE_ID;
}
