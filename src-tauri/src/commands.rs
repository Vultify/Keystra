//! Tauri commands the screens call. Keyboard work goes through the keyboard worker and profile
//! changes through `ProfilesState::change`, both off the window's main thread.

use serde::{Serialize, Serializer};
use tauri::{AppHandle, State};

use crate::keyboard::{KeyboardError, KeyboardHandle, KeyboardStatus, Snapshot};
use crate::profiles::{Profile, ProfileError, ProfileList, ProfilesState};

#[tauri::command]
pub fn keyboard_status(keyboard: State<'_, KeyboardHandle>) -> KeyboardStatus {
    keyboard.status()
}

#[tauri::command]
pub async fn read_keyboard(keyboard: State<'_, KeyboardHandle>) -> Result<Snapshot, KeyboardError> {
    read_all(&keyboard).await
}

#[tauri::command]
pub fn list_profiles(profiles: State<'_, ProfilesState>) -> ProfileList {
    profiles.list()
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    profiles: State<'_, ProfilesState>,
    name: String,
    copy_from: Option<u32>,
) -> Result<Profile, ProfileError> {
    profiles.change(&app, |store| store.create(&name, copy_from))
}

/// Reads the keyboard first and only then adds the profile, so a failed read creates nothing.
/// Read-only toward the keyboard.
#[tauri::command]
pub async fn create_profile_from_keyboard(
    app: AppHandle,
    keyboard: State<'_, KeyboardHandle>,
    profiles: State<'_, ProfilesState>,
    name: String,
) -> Result<Profile, SaveError> {
    let snapshot = read_all(&keyboard).await?;
    Ok(profiles.change(&app, |store| store.create_with(&name, |profile| profile.take_reading(&snapshot)))?)
}

#[tauri::command]
pub async fn rename_profile(
    app: AppHandle,
    profiles: State<'_, ProfilesState>,
    id: u32,
    name: String,
) -> Result<Profile, ProfileError> {
    profiles.change(&app, |store| store.rename(id, &name))
}

#[tauri::command]
pub async fn update_profile(
    app: AppHandle,
    profiles: State<'_, ProfilesState>,
    profile: Profile,
) -> Result<Profile, ProfileError> {
    profiles.change(&app, |store| store.update(profile))
}

#[tauri::command]
pub async fn delete_profile(app: AppHandle, profiles: State<'_, ProfilesState>, id: u32) -> Result<(), ProfileError> {
    profiles.change(&app, |store| store.delete(id))
}

/// Reads the keyboard and saves its current lighting, polling rate and onboard profile into a
/// Keystra profile. Read-only toward the keyboard.
#[tauri::command]
pub async fn save_keyboard_to_profile(
    app: AppHandle,
    keyboard: State<'_, KeyboardHandle>,
    profiles: State<'_, ProfilesState>,
    id: u32,
) -> Result<Profile, SaveError> {
    let snapshot = read_all(&keyboard).await?;
    Ok(profiles.change(&app, |store| {
        let mut profile = store.get(id)?.clone();
        profile.take_reading(&snapshot);
        store.update(profile)
    })?)
}

/// Runs a full read on a blocking thread, since the worker answers over a channel.
async fn read_all(keyboard: &KeyboardHandle) -> Result<Snapshot, KeyboardError> {
    let keyboard = keyboard.clone();
    tauri::async_runtime::spawn_blocking(move || keyboard.read_all())
        .await
        .map_err(|_| KeyboardError::WorkerStopped)?
}

/// Saving from the keyboard can fail on either side.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error(transparent)]
    Keyboard(#[from] KeyboardError),
    #[error(transparent)]
    Profile(#[from] ProfileError),
}

impl Serialize for SaveError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
