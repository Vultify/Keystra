use serde::{Serialize, Serializer};

use crate::keyboard::KeyboardError;
use crate::profiles::ProfileError;

/// Errors from commands, which can fail on the keyboard, the profiles or the Windows startup setting.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Keyboard(#[from] KeyboardError),
    #[error(transparent)]
    Profile(#[from] ProfileError),
    #[error("couldn't change the Windows startup setting: {0}")]
    Startup(#[from] auto_launch::Error),
}

// Commands return errors to the screens as plain messages.
impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
