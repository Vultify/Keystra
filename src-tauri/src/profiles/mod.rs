//! Saved profiles: the data model and the file store that keeps them on the PC.

mod error;
mod model;
mod store;

use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Emitter};

pub use error::ProfileError;
pub use model::{Profile, ProfileList};
#[cfg(test)]
pub use model::{MAX_NAME_LEN, ONBOARD_SLOTS};
pub use store::{FILE_NAME, ProfileStore};

/// Event the screens listen to for profile changes; the payload is a [`ProfileList`].
pub const PROFILES_EVENT: &str = "profiles-changed";

/// The one profile store, shared by the commands (and later the tray and game watcher).
pub struct ProfilesState(Mutex<ProfileStore>);

impl ProfilesState {
    pub fn new(store: ProfileStore) -> Self {
        Self(Mutex::new(store))
    }

    pub fn list(&self) -> ProfileList {
        self.lock().list()
    }

    /// The only way to change profiles: runs `edit` on the store, then announces the new list with
    /// PROFILES_EVENT after the lock is released, so a listener can safely read the store.
    pub fn change<T>(
        &self,
        app: &AppHandle,
        edit: impl FnOnce(&mut ProfileStore) -> Result<T, ProfileError>,
    ) -> Result<T, ProfileError> {
        let (result, list) = {
            let mut store = self.lock();
            let result = edit(&mut store)?;
            (result, store.list())
        };
        if let Err(err) = app.emit(PROFILES_EVENT, list) {
            log::warn!("could not send profiles to the window: {err}");
        }
        Ok(result)
    }

    fn lock(&self) -> MutexGuard<'_, ProfileStore> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
