//! Tauri commands the screens call. Keyboard work and anything that applies a profile run off
//! the window's main thread; profile changes go through `ProfilesState::change`.

use tauri::{AppHandle, State};

use crate::apply::{self, SaveOutcome};
use crate::error::CommandError;
use crate::keyboard::{KeyboardError, KeyboardHandle, KeyboardStatus, Snapshot};
use crate::profiles::{Profile, ProfileError, ProfileList, ProfilesState};

#[tauri::command]
pub fn keyboard_status(keyboard: State<'_, KeyboardHandle>) -> KeyboardStatus {
    keyboard.status()
}

#[tauri::command]
pub async fn read_keyboard(keyboard: State<'_, KeyboardHandle>) -> Result<Snapshot, KeyboardError> {
    let keyboard = keyboard.inner().clone();
    blocking(move || keyboard.read_all()).await
}

/// Sends a profile to the keyboard and makes it the active profile.
#[tauri::command]
pub async fn apply_profile(app: AppHandle, id: u32) -> Result<Snapshot, CommandError> {
    blocking(move || apply::apply_profile(&app, id)).await
}

/// Saves a profile's settings, and sends them to the keyboard when it's the active profile.
#[tauri::command]
pub async fn save_profile(app: AppHandle, profile: Profile) -> Result<SaveOutcome, CommandError> {
    blocking(move || Ok(apply::save_profile(&app, profile)?)).await
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
) -> Result<Profile, CommandError> {
    let keyboard = keyboard.inner().clone();
    let snapshot = blocking(move || keyboard.read_all()).await?;
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
) -> Result<Profile, CommandError> {
    let keyboard = keyboard.inner().clone();
    let snapshot = blocking(move || keyboard.read_all()).await?;
    Ok(profiles.change(&app, |store| {
        let mut profile = store.get(id)?.clone();
        profile.take_reading(&snapshot);
        store.update(profile)
    })?)
}

/// Runs blocking work (the keyboard worker answers over a channel) on a background thread.
async fn blocking<T, E>(job: impl FnOnce() -> Result<T, E> + Send + 'static) -> Result<T, E>
where
    T: Send + 'static,
    E: From<KeyboardError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|_| E::from(KeyboardError::WorkerStopped))?
}
