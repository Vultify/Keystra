import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { PROFILES_EVENT } from "./bindings/constants";
import type { Profile } from "./bindings/Profile";
import type { ProfileList } from "./bindings/ProfileList";
import type { SaveOutcome } from "./bindings/SaveOutcome";
import type { Snapshot } from "./bindings/Snapshot";

// Types and constants are generated from the Rust side by `cargo test` (see src/lib/bindings).
export { MAX_NAME_LEN, ONBOARD_SLOTS, POLLING_RATES } from "./bindings/constants";
export type { LightingSettings } from "./bindings/LightingSettings";
export type { Profile } from "./bindings/Profile";
export type { ProfileList } from "./bindings/ProfileList";
export type { SaveOutcome } from "./bindings/SaveOutcome";

export function listProfiles(): Promise<ProfileList> {
  return invoke<ProfileList>("list_profiles");
}

export function createProfile(name: string, copyFrom?: number): Promise<Profile> {
  return invoke<Profile>("create_profile", { name, copyFrom: copyFrom ?? null });
}

/** Reads the keyboard, then adds a profile holding its settings. Creates nothing if the read fails. */
export function createProfileFromKeyboard(name: string): Promise<Profile> {
  return invoke<Profile>("create_profile_from_keyboard", { name });
}

export function renameProfile(id: number, name: string): Promise<Profile> {
  return invoke<Profile>("rename_profile", { id, name });
}

/**
 * Saves a profile's settings, and sends them to the keyboard when it's the active profile. A
 * failed send comes back in `applyError`; the save itself still worked.
 */
export function saveProfile(profile: Profile): Promise<SaveOutcome> {
  return invoke<SaveOutcome>("save_profile", { profile });
}

export function deleteProfile(id: number): Promise<void> {
  return invoke<void>("delete_profile", { id });
}

/**
 * Sends a profile's onboard profile, polling rate and lighting to the keyboard and makes it the
 * active profile. The new settings also arrive through `onKeyboardSettings`.
 */
export function applyProfile(id: number): Promise<Snapshot> {
  return invoke<Snapshot>("apply_profile", { id });
}

/** Reads the keyboard and saves its lighting, polling rate and onboard profile into a profile. */
export function saveKeyboardToProfile(id: number): Promise<Profile> {
  return invoke<Profile>("save_keyboard_to_profile", { id });
}

export function onProfilesChanged(handler: (list: ProfileList) => void): Promise<UnlistenFn> {
  return listen<ProfileList>(PROFILES_EVENT, (event) => handler(event.payload));
}
