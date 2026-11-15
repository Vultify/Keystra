//! Sending profiles to the keyboard: the one path the screens, the tray and the game watcher use.
//! Everything here blocks on the keyboard worker, so callers run it off the main thread.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::error::CommandError;
use crate::keyboard::{ApplyRequest, KeyboardError, KeyboardHandle, STATUS_EVENT, Snapshot};
use crate::profiles::{Profile, ProfileError, ProfilesState};

/// Event the screens listen to for the keyboard's settings after Keystra changed them; the
/// payload is a [`Snapshot`].
pub const SETTINGS_EVENT: &str = "keyboard-settings";

/// Counts profile switches, so a save that raced one can tell and put the switched-to profile back.
static SWITCHES: AtomicU64 = AtomicU64::new(0);

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

/// Makes a profile the one in use and sends it to the keyboard. It's marked in use first, so an
/// edit racing the switch sees it. Returns `None` when a newer change took this one's place.
pub fn apply_profile(app: &AppHandle, id: u32) -> Result<Option<Snapshot>, CommandError> {
    let profiles = app.state::<ProfilesState>();
    let profile = profiles.get_for_apply(id)?;
    SWITCHES.fetch_add(1, Ordering::SeqCst);
    profiles.change(app, |store| store.set_active(id))?;
    send(app, &profile)
}

/// Saves a profile, and sends it to the keyboard when it's the one in use and the keyboard is
/// connected, so editing the active profile changes the keyboard too.
pub fn save_profile(app: &AppHandle, profile: Profile) -> Result<SaveOutcome, ProfileError> {
    let profiles = app.state::<ProfilesState>();
    let saved = profiles.change(app, |store| store.update(profile))?;
    let in_use = profiles.list().active_id == Some(saved.id);
    if !in_use || !keyboard_connected(app) {
        return Ok(SaveOutcome { profile: saved, apply_error: None });
    }
    let switches = SWITCHES.load(Ordering::SeqCst);
    let sent = send(app, &saved);
    // A switch that started meanwhile wins: send the profile now in use so the keyboard matches it.
    if SWITCHES.load(Ordering::SeqCst) != switches {
        resend_active(app);
        return Ok(SaveOutcome { profile: saved, apply_error: None });
    }
    Ok(SaveOutcome { profile: saved, apply_error: sent.err().map(|err| err.to_string()) })
}

/// Whether the keyboard is connected right now. False until the keyboard worker has started.
pub fn keyboard_connected(app: &AppHandle) -> bool {
    app.try_state::<KeyboardHandle>().is_some_and(|keyboard| keyboard.is_connected())
}

/// Re-sends the profile in use each time the keyboard connects, so changes made while it was
/// unplugged arrive. Only settings that differ are written.
pub fn follow_connections(app: &AppHandle) {
    let handle = app.clone();
    let was_connected = Arc::new(AtomicBool::new(false));
    app.listen(STATUS_EVENT, move |_| {
        let connected = keyboard_connected(&handle);
        if connected && !was_connected.swap(true, Ordering::SeqCst) {
            // Status events arrive on the keyboard worker's own thread, which must stay free to do the work.
            let handle = handle.clone();
            tauri::async_runtime::spawn_blocking(move || resend_active(&handle));
        } else if !connected {
            was_connected.store(false, Ordering::SeqCst);
        }
    });
}

/// Sends the profile in use, if there is one and the profiles can be used.
fn resend_active(app: &AppHandle) {
    let profiles = app.state::<ProfilesState>();
    let Some(id) = profiles.list().active_id else { return };
    let result = profiles.get_for_apply(id).map_err(CommandError::from).and_then(|profile| send(app, &profile));
    if let Err(err) = result {
        log::warn!("couldn't send the profile in use to the keyboard: {err}");
    }
}

/// Sends a profile's settings and announces what the keyboard holds afterwards, even when a
/// later step failed and only part of the profile got through.
fn send(app: &AppHandle, profile: &Profile) -> Result<Option<Snapshot>, CommandError> {
    let keyboard = app.state::<KeyboardHandle>();
    match keyboard.apply(ApplyRequest::from(profile)) {
        Ok(snapshot) => {
            app.state::<ProfilesState>().set_unsent(app, None);
            announce(app, &snapshot);
            Ok(Some(snapshot))
        }
        // The newer change's caller announces what the keyboard ends up holding.
        Err(KeyboardError::Superseded) => Ok(None),
        Err(err) => {
            app.state::<ProfilesState>().set_unsent(app, Some(profile.id));
            if let Ok(snapshot) = keyboard.read_all() {
                announce(app, &snapshot);
            }
            Err(err.into())
        }
    }
}

fn announce(app: &AppHandle, snapshot: &Snapshot) {
    if let Err(err) = app.emit(SETTINGS_EVENT, snapshot) {
        log::warn!("could not send keyboard settings to the window: {err}");
    }
}
