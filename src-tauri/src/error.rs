use serde::{Serialize, Serializer};

use crate::keyboard::KeyboardError;
use crate::profiles::ProfileError;

/// Work that touches both the keyboard and the profiles can fail on either side.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Keyboard(#[from] KeyboardError),
    #[error(transparent)]
    Profile(#[from] ProfileError),
}

// Commands return errors to the screens as plain messages.
impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
