//! Keystra's JSON files: loading them by the same rules everywhere, and saving them so a crash
//! mid-save never leaves a half-written file behind.

use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// One upgrade step: `steps[n]` turns a version n + 1 file into version n + 2.
pub type Upgrade = fn(Value) -> Result<Value, String>;

/// Why a saved file couldn't be loaded. Only `NotJson` may be set aside and replaced; for the
/// others the file is left exactly as it is.
#[derive(Debug)]
pub enum LoadError {
    NotJson(String),
    TooNew(u32),
    Unusable(String),
}

/// Reads a file saved with a `schemaVersion`, runs the upgrade steps it needs and parses it.
/// Returns `None` when the file doesn't exist yet.
pub fn load_versioned<T: DeserializeOwned>(path: &Path, version: u32, upgrades: &[Upgrade]) -> Result<Option<T>, LoadError> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("the file");
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(LoadError::Unusable(format!("couldn't read {name} ({err})"))),
    };
    let mut raw: Value = serde_json::from_slice(&bytes).map_err(|err| LoadError::NotJson(err.to_string()))?;
    let saved = raw
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .filter(|&v| v > 0)
        .ok_or_else(|| LoadError::Unusable(format!("{name} has no usable schemaVersion")))?;
    if saved > version {
        return Err(LoadError::TooNew(saved));
    }
    for step in upgrades.iter().skip(saved as usize - 1) {
        raw = step(raw).map_err(LoadError::Unusable)?;
    }
    if let Some(fields) = raw.as_object_mut() {
        fields.insert("schemaVersion".into(), version.into());
    }
    serde_json::from_value(raw)
        .map(Some)
        .map_err(|err| LoadError::Unusable(format!("{name} isn't laid out as expected ({err})")))
}

/// Writes `value` to a temporary file, flushes it to disk, then swaps it in place of `path`.
pub fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    let mut out = File::create(&temp)?;
    out.write_all(json.as_bytes())?;
    out.sync_all()?;
    drop(out);
    fs::rename(&temp, path)
}

/// Moves a damaged file aside as `<stem>.damaged-<seconds>.json` and returns the new name.
pub fn set_aside_damaged(path: &Path) -> io::Result<String> {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let name = format!("{stem}.damaged-{stamp}.json");
    fs::rename(path, path.with_file_name(&name))?;
    Ok(name)
}

/// A fresh folder under the system temp folder for one test, removed when the test ends.
#[cfg(test)]
pub struct TempDir(pub std::path::PathBuf);

#[cfg(test)]
impl TempDir {
    pub fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("keystra-test-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn join(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
}

#[cfg(test)]
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
