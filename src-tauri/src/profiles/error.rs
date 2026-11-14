use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("{0}")]
    Invalid(String),

    #[error("a profile named \"{0}\" already exists")]
    NameTaken(String),

    #[error("profile not found")]
    NotFound,

    #[error("the last profile can't be deleted")]
    LastProfile,

    #[error("switch to another profile before deleting the one in use")]
    InUse,

    #[error("the profiles file was saved by a newer Keystra (format {0}), so it's left untouched")]
    TooNew(u32),

    #[error("profiles can't be changed right now: {0}")]
    ReadOnly(String),

    #[error("the profiles file is damaged: {0}")]
    Corrupt(String),

    #[error("couldn't save profiles: {0}")]
    Io(String),
}

// Commands return errors to the screens as plain messages.
impl Serialize for ProfileError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
