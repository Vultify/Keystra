import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { PROFILES_EVENT } from "./bindings/constants";
import type { Profile } from "./bindings/Profile";
import type { ProfileList } from "./bindings/ProfileList";
import type { SaveOutcome } from "./bindings/SaveOutcome";

// Types and constants are generated from the Rust side by `cargo test` (see src/lib/bindings).
export { MAX_NAME_LEN, ONBOARD_SLOTS, POLLING_RATES } from "./bindings/constants";
export type { LightingSettings } from "./bindings/LightingSettings";
export type { Profile } from "./bindings/Profile";
export type { ProfileList } from "./bindings/ProfileList";
export type { SaveOutcome } from "./bindings/SaveOutcome";

export function listProfiles(): Promise<ProfileList> {
  return invoke<ProfileList>("list_profiles");
}

export function createProfile(name: string): Promise<Profile> {
  return invoke<Profile>("create_profile", { name });
}

/**
 * Reads the keyboard, then adds a profile holding its settings. Creates nothing if the read fails.
 * Without a name it gets a free "My keyboard" one.
 */
export function createProfileFromKeyboard(name?: string): Promise<Profile> {
  return invoke<Profile>("create_profile_from_keyboard", { name: name ?? null });
}

/** Copies a profile under a free "<name> copy" name. */
export function duplicateProfile(id: number): Promise<Profile> {
  return invoke<Profile>("duplicate_profile", { id });
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
 * Makes a profile the one in use and sends its onboard profile, polling rate and lighting to the
 * keyboard. What the keyboard holds afterwards arrives through `onKeyboardSettings`.
 */
export function applyProfile(id: number): Promise<void> {
  return invoke<void>("apply_profile", { id });
}

/** Reads the keyboard and saves its lighting, polling rate and onboard profile into a profile. */
export function saveKeyboardToProfile(id: number): Promise<Profile> {
  return invoke<Profile>("save_keyboard_to_profile", { id });
}

export function onProfilesChanged(handler: (list: ProfileList) => void): Promise<UnlistenFn> {
  return listen<ProfileList>(PROFILES_EVENT, (event) => handler(event.payload));
}
