//! Saving Keystra's JSON files so a crash mid-save never leaves a half-written file behind.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

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
