//! Saved profiles: the data model and the file store that keeps them on the PC.

mod error;
mod model;
mod store;

use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Emitter};

pub use error::ProfileError;
pub use model::{Profile, ProfileList};
#[cfg(test)]
pub use model::MAX_NAME_LEN;
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

    /// A profile to send to the keyboard. Refused while the store is read-only, since its
    /// placeholder profile isn't the user's and the switch couldn't be remembered anyway.
    pub fn get_for_apply(&self, id: u32) -> Result<Profile, ProfileError> {
        let store = self.lock();
        store.check_writable()?;
        store.get(id).cloned()
    }

    /// The only way to change profiles: runs `edit` on the store, then, if anything changed,
    /// announces the new list with PROFILES_EVENT after the lock is released.
    pub fn change<T>(
        &self,
        app: &AppHandle,
        edit: impl FnOnce(&mut ProfileStore) -> Result<T, ProfileError>,
    ) -> Result<T, ProfileError> {
        let (result, changed) = {
            let mut store = self.lock();
            let revision = store.revision();
            let result = edit(&mut store)?;
            let changed = (store.revision() != revision).then(|| store.list());
            (result, changed)
        };
        if let Some(list) = changed
            && let Err(err) = app.emit(PROFILES_EVENT, list)
        {
            log::warn!("could not send profiles to the window: {err}");
        }
        Ok(result)
    }

    fn lock(&self) -> MutexGuard<'_, ProfileStore> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
