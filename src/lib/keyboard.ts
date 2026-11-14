import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { STATUS_EVENT } from "./bindings/constants";
import type { KeyboardStatus } from "./bindings/KeyboardStatus";
import type { Snapshot } from "./bindings/Snapshot";

// Types and constants are generated from the Rust side by `cargo test` (see src/lib/bindings).
export { MAX_LEVEL } from "./bindings/constants";
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
