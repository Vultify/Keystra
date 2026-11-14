import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { PROFILES_EVENT } from "./bindings/constants";
import type { Profile } from "./bindings/Profile";
import type { ProfileList } from "./bindings/ProfileList";

// Types and constants are generated from the Rust side by `cargo test` (see src/lib/bindings).
export { MAX_NAME_LEN, ONBOARD_SLOTS, POLLING_RATES } from "./bindings/constants";
export type { LightingSettings } from "./bindings/LightingSettings";
export type { Profile } from "./bindings/Profile";
export type { ProfileList } from "./bindings/ProfileList";

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

export function updateProfile(profile: Profile): Promise<Profile> {
  return invoke<Profile>("update_profile", { profile });
}

export function deleteProfile(id: number): Promise<void> {
  return invoke<void>("delete_profile", { id });
}

/** Reads the keyboard and saves its lighting, polling rate and onboard profile into a profile. */
export function saveKeyboardToProfile(id: number): Promise<Profile> {
  return invoke<Profile>("save_keyboard_to_profile", { id });
}

export function onProfilesChanged(handler: (list: ProfileList) => void): Promise<UnlistenFn> {
  return listen<ProfileList>(PROFILES_EVENT, (event) => handler(event.payload));
}
