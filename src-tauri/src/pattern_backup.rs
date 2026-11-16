//! A copy of the keyboard's per-key color patterns on this PC, so a keyboard reset can't lose them.
//! Kept apart from the profiles: the patterns live on the keyboard, and a profile only picks one.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::files;
use crate::keyboard::{KeyboardHandle, LIGHT_COUNT, PATTERN_SLOTS, Pattern};

pub const FILE_NAME: &str = "patterns.json";
const SCHEMA_VERSION: u32 = 1;

/// Upgrade steps for older saved formats: `UPGRADES[n]` turns a version n + 1 file into version
/// n + 2. Its length is tied to SCHEMA_VERSION, so bumping the version without a step won't build.
const UPGRADES: [fn(Value) -> Value; SCHEMA_VERSION as usize - 1] = [];

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupFile {
    schema_version: u32,
    /// One entry per slot; `null` until Keystra has seen that pattern.
    patterns: Vec<Option<Pattern>>,
}

impl BackupFile {
    fn empty() -> Self {
        Self { schema_version: SCHEMA_VERSION, patterns: vec![None; usize::from(PATTERN_SLOTS)] }
    }
}

/// A saved pattern as the keyboard holds it, and Keystra's copy when that differs.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct PatternReading {
    pub keyboard: Pattern,
    /// Keystra's copy, only when it differs from the keyboard (after a reset, or a change in MagCrate).
    pub saved_copy: Option<Pattern>,
}

struct Copies {
    file: BackupFile,
    /// False when the saved file couldn't be used; it's then left exactly as it was.
    writable: bool,
}

pub struct PatternBackup {
    path: PathBuf,
    copies: Mutex<Copies>,
}

impl PatternBackup {
    /// Opens the copy like the profiles store does: only bytes that aren't JSON are set aside, and a
    /// file that can't be read or used (including a newer format) is left alone and never written.
    pub fn open(path: PathBuf) -> Self {
        let (file, writable) = match load(&path) {
            Ok(file) => (file, true),
            Err(Load::Missing) => (BackupFile::empty(), true),
            Err(Load::NotJson) => match files::set_aside_damaged(&path) {
                Ok(name) => {
                    log::warn!("{FILE_NAME} wasn't valid JSON; kept it as {name}");
                    (BackupFile::empty(), true)
                }
                Err(err) => {
                    log::warn!("{FILE_NAME} wasn't valid JSON and couldn't be moved aside ({err}); leaving it alone");
                    (BackupFile::empty(), false)
                }
            },
            Err(Load::Unusable(reason)) => {
                log::warn!("leaving {FILE_NAME} alone: {reason}");
                (BackupFile::empty(), false)
            }
        };
        Self { path, copies: Mutex::new(Copies { file, writable }) }
    }

    pub fn get(&self, slot: u8) -> Option<Pattern> {
        self.lock().file.patterns.get(usize::from(slot)).cloned().flatten()
    }

    /// Slots Keystra has no copy of yet.
    pub fn missing(&self) -> Vec<u8> {
        (0..PATTERN_SLOTS).filter(|&slot| self.get(slot).is_none()).collect()
    }

    /// What the screens show after a read: the first time a slot is seen its pattern becomes the
    /// copy; after that a read never replaces the copy, and a copy that differs is offered back.
    pub fn reading(&self, slot: u8, keyboard: Pattern) -> PatternReading {
        match self.get(slot) {
            None => {
                self.remember(slot, &keyboard);
                PatternReading { keyboard, saved_copy: None }
            }
            Some(copy) => {
                let saved_copy = (copy != keyboard).then_some(copy);
                PatternReading { keyboard, saved_copy }
            }
        }
    }

    /// Keeps `pattern` as the copy of `slot`, saving the file when it changed.
    pub fn remember(&self, slot: u8, pattern: &Pattern) {
        let mut copies = self.lock();
        let Some(entry) = copies.file.patterns.get_mut(usize::from(slot)) else { return };
        if entry.as_ref() == Some(pattern) {
            return;
        }
        *entry = Some(pattern.clone());
        if !copies.writable {
            return;
        }
        if let Err(err) = files::write_json(&self.path, &copies.file) {
            log::warn!("couldn't save the pattern copy: {err}");
        }
    }

    fn lock(&self) -> MutexGuard<'_, Copies> {
        self.copies.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Copies any pattern Keystra hasn't seen yet, so a reset can't lose one that was only ever
/// painted in MagCrate. Reads only; nothing is written to the keyboard.
pub fn copy_missing(app: &AppHandle) {
    let (Some(keyboard), Some(backup)) = (app.try_state::<KeyboardHandle>(), app.try_state::<PatternBackup>()) else {
        return;
    };
    for slot in backup.missing() {
        match keyboard.read_pattern(slot) {
            Ok(pattern) => backup.remember(slot, &pattern),
            Err(err) => {
                log::warn!("couldn't copy pattern {}: {err}", slot + 1);
                return;
            }
        }
    }
}

enum Load {
    Missing,
    NotJson,
    Unusable(String),
}

fn load(path: &std::path::Path) -> Result<BackupFile, Load> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Err(Load::Missing),
        Err(err) => return Err(Load::Unusable(format!("couldn't read it ({err})"))),
    };
    let mut raw: Value = serde_json::from_slice(&bytes).map_err(|_| Load::NotJson)?;
    let version = raw
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .filter(|&v| v > 0)
        .ok_or_else(|| Load::Unusable("it has no usable schemaVersion".into()))?;
    if version > SCHEMA_VERSION {
        return Err(Load::Unusable(format!("it was saved by a newer Keystra (format {version})")));
    }
    for step in UPGRADES.iter().skip(version as usize - 1) {
        raw = step(raw);
    }
    if let Some(fields) = raw.as_object_mut() {
        fields.insert("schemaVersion".into(), SCHEMA_VERSION.into());
    }
    let file: BackupFile =
        serde_json::from_value(raw).map_err(|err| Load::Unusable(format!("it isn't laid out as expected ({err})")))?;
    Ok(sanitize(file))
}

/// Drops entries of the wrong size and pads or trims to one entry per slot.
fn sanitize(mut file: BackupFile) -> BackupFile {
    file.patterns.resize(usize::from(PATTERN_SLOTS), None);
    for entry in &mut file.patterns {
        if entry.as_ref().is_some_and(|p| p.0.len() != LIGHT_COUNT) {
            *entry = None;
        }
    }
    file
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("keystra-patterns-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join(FILE_NAME)
    }

    fn painted(light: usize, color: &str) -> Pattern {
        let mut colors = vec!["#000000".to_string(); LIGHT_COUNT];
        colors[light] = color.into();
        Pattern(colors)
    }

    #[test]
    fn remembers_patterns_across_restarts() {
        let path = temp_path("restart");
        let backup = PatternBackup::open(path.clone());
        assert_eq!(backup.get(0), None);
        backup.remember(0, &painted(14, "#FF0000"));
        assert_eq!(PatternBackup::open(path).get(0), Some(painted(14, "#FF0000")));
    }

    #[test]
    fn the_first_read_keeps_a_copy_and_later_reads_never_replace_it() {
        let backup = PatternBackup::open(temp_path("reading"));
        let first = backup.reading(2, painted(14, "#FF0000"));
        assert_eq!(first.saved_copy, None);
        assert_eq!(backup.get(2), Some(painted(14, "#FF0000")));

        // The keyboard was reset: the copy stays and is offered back.
        let after_reset = backup.reading(2, painted(14, "#000000"));
        assert_eq!(after_reset.saved_copy, Some(painted(14, "#FF0000")));
        assert_eq!(backup.get(2), Some(painted(14, "#FF0000")));

        // Keeping the keyboard's version makes it the copy, and the offer goes away.
        backup.remember(2, &painted(14, "#000000"));
        assert_eq!(backup.reading(2, painted(14, "#000000")).saved_copy, None);
    }

    #[test]
    fn lists_the_slots_without_a_copy() {
        let backup = PatternBackup::open(temp_path("missing"));
        backup.remember(1, &painted(0, "#FFFFFF"));
        assert_eq!(backup.missing(), vec![0, 2, 3, 4]);
    }

    #[test]
    fn a_file_that_is_not_json_is_set_aside() {
        let path = temp_path("damaged");
        fs::write(&path, b"not json").unwrap();
        let backup = PatternBackup::open(path.clone());
        assert_eq!(backup.get(0), None);
        let kept = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().starts_with("patterns.damaged-"));
        assert!(kept);
    }

    #[test]
    fn a_newer_format_is_left_alone_and_never_written() {
        let path = temp_path("newer");
        let newer = br#"{"schemaVersion":9,"patterns":[]}"#;
        fs::write(&path, newer).unwrap();
        let backup = PatternBackup::open(path.clone());
        backup.remember(0, &painted(0, "#FF0000"));
        assert_eq!(fs::read(&path).unwrap(), newer);
    }

    #[test]
    fn a_file_laid_out_wrong_is_left_alone() {
        let path = temp_path("layout");
        let odd = br#"{"schemaVersion":1,"patterns":"nope"}"#;
        fs::write(&path, odd).unwrap();
        let backup = PatternBackup::open(path.clone());
        backup.remember(0, &painted(0, "#FF0000"));
        assert_eq!(fs::read(&path).unwrap(), odd);
    }

    #[test]
    fn wrong_sized_entries_are_dropped() {
        let path = temp_path("sized");
        fs::write(&path, br##"{"schemaVersion":1,"patterns":[["#FF0000"]]}"##).unwrap();
        let backup = PatternBackup::open(path);
        assert_eq!(backup.get(0), None);
        assert_eq!(backup.get(PATTERN_SLOTS), None);
    }
}
