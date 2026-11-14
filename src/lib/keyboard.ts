import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Shapes mirror the Rust types in src-tauri/src/keyboard.

export type DeviceInfo = {
  deviceId: number;
  firmwareVersion: number;
  model: string | null;
};

export type KeyboardStatus =
  | { state: "disconnected" }
  | { state: "connected"; info: DeviceInfo; magcrateOpen: boolean }
  | { state: "unsupported"; deviceId: number }
  | { state: "error"; message: string };

export type Lighting = {
  modeId: number;
  modeName: string | null;
  brightness: number;
  speed: number;
  direction: number;
  rainbow: boolean;
  /** `null` when the effect picks its own colors. */
  color: string | null;
};

export type Exchange = {
  command: string;
  name: string;
  request: number[];
  reply: number[];
};

export type Snapshot = {
  info: DeviceInfo;
  profile: number;
  pollingRate: number | null;
  lighting: Lighting;
  exchanges: Exchange[];
};

const STATUS_EVENT = "keyboard-status";

/** Brightness and speed both run from 0 to this value. */
export const LEVEL_MAX = 4;

export function getKeyboardStatus(): Promise<KeyboardStatus> {
  return invoke<KeyboardStatus>("keyboard_status");
}

export function readKeyboard(): Promise<Snapshot> {
  return invoke<Snapshot>("read_keyboard");
}

export function onKeyboardStatus(handler: (status: KeyboardStatus) => void): Promise<UnlistenFn> {
  return listen<KeyboardStatus>(STATUS_EVENT, (event) => handler(event.payload));
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
