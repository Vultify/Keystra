use serde::{Serialize, Serializer};

/// Everything that can go wrong while talking to the keyboard.
#[derive(Clone, Debug, thiserror::Error)]
pub enum KeyboardError {
    #[error("keyboard is not connected")]
    NotConnected,

    #[error("command 0x{0:02X} is on the never-send list")]
    Blocked(u8),

    #[error("USB error: {0}")]
    Hid(String),

    #[error("unexpected reply to {command}: {reason}")]
    BadReply { command: &'static str, reason: String },

    #[error("the keyboard worker has stopped")]
    WorkerStopped,

    #[error("{0}")]
    InvalidSetting(String),

    #[error("the keyboard didn't keep the new {0}; it may be busy with another app")]
    NotConfirmed(&'static str),

    #[error("a newer change replaced this one before it was sent")]
    Superseded,
}

impl From<hidapi::HidError> for KeyboardError {
    fn from(err: hidapi::HidError) -> Self {
        Self::Hid(err.to_string())
    }
}

// Commands return errors to the screens as plain messages.
impl Serialize for KeyboardError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
