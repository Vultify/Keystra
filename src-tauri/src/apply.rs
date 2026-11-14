//! Sending profiles to the keyboard: the one path the screens, the tray and the game watcher use.
//! Everything here blocks on the keyboard worker, so callers run it off the main thread.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::CommandError;
use crate::keyboard::{ApplyRequest, KeyboardError, KeyboardHandle, Snapshot};
use crate::profiles::{Profile, ProfileError, ProfilesState};

/// Event the screens listen to for the keyboard's settings after Keystra changed them; the
/// payload is a [`Snapshot`].
pub const SETTINGS_EVENT: &str = "keyboard-settings";

impl From<&Profile> for ApplyRequest {
    fn from(profile: &Profile) -> Self {
        ApplyRequest {
            onboard_slot: Some(profile.onboard_slot),
            polling_rate_hz: Some(profile.polling_rate_hz),
            lighting: Some(profile.lighting.clone()),
        }
    }
}

/// The result of saving a profile. A save that worked never reads as failed: if sending it to
/// the keyboard failed, that's reported in `apply_error` instead.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SaveOutcome {
    pub profile: Profile,
    pub apply_error: Option<String>,
}

/// Sends a profile to the keyboard and makes it the active profile. Announces what the keyboard
/// holds afterwards, even when a later step failed and only part of the profile got through.
pub fn apply_profile(app: &AppHandle, id: u32) -> Result<Snapshot, CommandError> {
    let keyboard = app.state::<KeyboardHandle>();
    let profiles = app.state::<ProfilesState>();
    let profile = profiles.get_for_apply(id)?;
    match keyboard.apply(ApplyRequest::from(&profile)) {
        Ok(snapshot) => {
            announce(app, &snapshot);
            if let Err(err) = profiles.change(app, |store| store.set_active(id)) {
                log::warn!("sent a profile to the keyboard but couldn't remember it as active: {err}");
            }
            Ok(snapshot)
        }
        // The newer change's caller announces the keyboard's settings and marks its profile active.
        Err(err @ KeyboardError::Superseded) => Err(err.into()),
        Err(err) => {
            if let Ok(snapshot) = keyboard.read_all() {
                announce(app, &snapshot);
            }
            Err(err.into())
        }
    }
}

/// Saves a profile, and sends it to the keyboard when it's the active one and the keyboard is
/// connected, so editing the active profile changes the keyboard too.
pub fn save_profile(app: &AppHandle, profile: Profile) -> Result<SaveOutcome, ProfileError> {
    let profiles = app.state::<ProfilesState>();
    let saved = profiles.change(app, |store| store.update(profile))?;
    let is_active = profiles.list().active_id == Some(saved.id);
    let apply_error = if is_active && app.state::<KeyboardHandle>().is_connected() {
        apply_profile(app, saved.id).err().map(|err| err.to_string())
    } else {
        None
    };
    Ok(SaveOutcome { profile: saved, apply_error })
}

fn announce(app: &AppHandle, snapshot: &Snapshot) {
    if let Err(err) = app.emit(SETTINGS_EVENT, snapshot) {
        log::warn!("could not send keyboard settings to the window: {err}");
    }
}
