//! Tauri commands the screens call. Keyboard work and anything that applies a profile run off
//! the window's main thread; profile changes go through `ProfilesState::change`.

use tauri::{AppHandle, State};

use crate::apply::{self, SaveOutcome};
use crate::error::CommandError;
use crate::keyboard::{KeyboardError, KeyboardHandle, KeyboardStatus, Snapshot};
use crate::profiles::{Profile, ProfileError, ProfileList, ProfilesState};
use crate::startup;

#[tauri::command]
pub fn keyboard_status(keyboard: State<'_, KeyboardHandle>) -> KeyboardStatus {
    keyboard.status()
}

#[tauri::command]
pub async fn read_keyboard(keyboard: State<'_, KeyboardHandle>) -> Result<Snapshot, KeyboardError> {
    let keyboard = keyboard.inner().clone();
    blocking(move || keyboard.read_all()).await
}

/// Makes a profile the one in use and sends it to the keyboard.
#[tauri::command]
pub async fn apply_profile(app: AppHandle, id: u32) -> Result<Option<Snapshot>, CommandError> {
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
) -> Result<Profile, ProfileError> {
    profiles.change(&app, |store| store.create(&name, None))
}

/// Reads the keyboard first and only then adds the profile, so a failed read creates nothing.
/// Without a name it gets a free "My keyboard" one. Read-only toward the keyboard.
#[tauri::command]
pub async fn create_profile_from_keyboard(
    app: AppHandle,
    keyboard: State<'_, KeyboardHandle>,
    profiles: State<'_, ProfilesState>,
    name: Option<String>,
) -> Result<Profile, CommandError> {
    let keyboard = keyboard.inner().clone();
    let snapshot = blocking(move || keyboard.read_all()).await?;
    Ok(profiles.change(&app, |store| {
        let name = name.unwrap_or_else(|| store.free_name("My keyboard", ""));
        store.create_with(&name, |profile| profile.take_reading(&snapshot))
    })?)
}

/// Copies a profile under a free "<name> copy" name.
#[tauri::command]
pub async fn duplicate_profile(app: AppHandle, profiles: State<'_, ProfilesState>, id: u32) -> Result<Profile, ProfileError> {
    profiles.change(&app, |store| store.duplicate(id))
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

/// Whether Windows starts Keystra (in the tray) when you sign in.
#[tauri::command]
pub fn start_with_windows() -> Result<bool, CommandError> {
    Ok(startup::is_enabled()?)
}

/// Turns starting with Windows on or off, then reports what Windows now has.
#[tauri::command]
pub fn set_start_with_windows(enabled: bool) -> Result<bool, CommandError> {
    Ok(startup::set_enabled(enabled)?)
}
