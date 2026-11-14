//! Reads and writes the profiles file. Only Rust touches it; the screens, the tray and the game
//! watcher all go through this store.

use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use super::error::ProfileError;
use super::model::{Profile, ProfileFile, ProfileList, SCHEMA_VERSION};

pub const FILE_NAME: &str = "profiles.json";

/// Upgrade steps for older saved formats: `UPGRADES[n]` turns a version n + 1 file into version
/// n + 2. The length is tied to SCHEMA_VERSION, so a version bump without its step doesn't build.
const UPGRADES: [fn(Value) -> Result<Value, ProfileError>; SCHEMA_VERSION as usize - 1] = [];

pub struct ProfileStore {
    path: PathBuf,
    file: ProfileFile,
    /// Set when the saved file couldn't be used. Changes are refused so that file is never overwritten.
    read_only: Option<String>,
}

impl ProfileStore {
    /// Opens the store, creating a first-run file if there is none. Only a file that can't be read
    /// as JSON is backed up and replaced; any other problem opens the store read-only.
    pub fn open(path: PathBuf) -> Self {
        match read_file(&path) {
            Ok(Some(file)) => Self { path, file, read_only: None },
            Ok(None) => {
                log::info!("no profiles yet, creating {FILE_NAME}");
                Self::start_fresh(path)
            }
            Err(ProfileError::Corrupt(reason)) => {
                log::warn!("{FILE_NAME} is damaged: {reason}");
                match backup_damaged(&path) {
                    Ok(backup) => {
                        log::warn!("kept the damaged file as {backup}");
                        Self::start_fresh(path)
                    }
                    Err(err) => Self::read_only(path, format!("the profiles file is damaged and couldn't be backed up ({err})")),
                }
            }
            Err(err) => Self::read_only(path, err.to_string()),
        }
    }

    fn start_fresh(path: PathBuf) -> Self {
        let store = Self { path, file: ProfileFile::fresh(), read_only: None };
        if let Err(err) = write_file(&store.path, &store.file) {
            log::warn!("{err}");
        }
        store
    }

    /// Shows a placeholder Default profile and refuses every change, leaving the saved file alone.
    fn read_only(path: PathBuf, reason: String) -> Self {
        log::warn!("opening profiles read-only: {reason}");
        Self { path, file: ProfileFile::fresh(), read_only: Some(reason) }
    }

    #[cfg(test)]
    fn profiles(&self) -> &[Profile] {
        &self.file.profiles
    }

    pub fn list(&self) -> ProfileList {
        ProfileList { profiles: self.file.profiles.clone(), read_only: self.read_only.clone() }
    }

    pub fn get(&self, id: u32) -> Result<&Profile, ProfileError> {
        self.file.profiles.iter().find(|p| p.id == id).ok_or(ProfileError::NotFound)
    }

    /// Adds a profile with the given name, copying another profile's settings or starting from
    /// the factory settings. A copy doesn't take the source's linked programs.
    pub fn create(&mut self, name: &str, copy_from: Option<u32>) -> Result<Profile, ProfileError> {
        let base = match copy_from {
            Some(id) => Some(self.get(id)?.clone()),
            None => None,
        };
        self.create_with(name, |profile| {
            if let Some(source) = base {
                profile.onboard_slot = source.onboard_slot;
                profile.lighting = source.lighting;
                profile.polling_rate_hz = source.polling_rate_hz;
            }
        })
    }

    /// Adds a factory-settings profile after `adjust` changes its settings, in a single save.
    pub fn create_with(&mut self, name: &str, adjust: impl FnOnce(&mut Profile)) -> Result<Profile, ProfileError> {
        self.change(|file| {
            let mut profile = Profile::factory(file.next_id, name);
            adjust(&mut profile);
            file.next_id += 1;
            file.profiles.push(profile.clone());
            Ok(profile)
        })
    }

    pub fn rename(&mut self, id: u32, name: &str) -> Result<Profile, ProfileError> {
        self.change(|file| {
            let profile = find_mut(file, id)?;
            profile.name = name.into();
            Ok(profile.clone())
        })
    }

    /// Replaces a profile's settings. The id must already exist.
    pub fn update(&mut self, profile: Profile) -> Result<Profile, ProfileError> {
        self.change(|file| {
            let slot = find_mut(file, profile.id)?;
            *slot = profile.clone();
            Ok(profile)
        })
    }

    pub fn delete(&mut self, id: u32) -> Result<(), ProfileError> {
        self.change(|file| {
            let before = file.profiles.len();
            file.profiles.retain(|p| p.id != id);
            match file.profiles.len() {
                0 => Err(ProfileError::LastProfile),
                n if n == before => Err(ProfileError::NotFound),
                _ => Ok(()),
            }
        })
    }

    /// Applies a change to a copy, checks the result, saves it, and only then keeps it. A failed
    /// check or save leaves both memory and disk as they were.
    fn change<T>(&mut self, edit: impl FnOnce(&mut ProfileFile) -> Result<T, ProfileError>) -> Result<T, ProfileError> {
        if let Some(reason) = &self.read_only {
            return Err(ProfileError::ReadOnly(reason.clone()));
        }
        let mut next = self.file.clone();
        let result = edit(&mut next)?;
        next.validate()?;
        write_file(&self.path, &next)?;
        self.file = next;
        Ok(result)
    }
}

fn find_mut(file: &mut ProfileFile, id: u32) -> Result<&mut Profile, ProfileError> {
    file.profiles.iter_mut().find(|p| p.id == id).ok_or(ProfileError::NotFound)
}

/// Reads the file, or `None` if it doesn't exist yet. Only bytes that aren't JSON count as `Corrupt`.
fn read_file(path: &Path) -> Result<Option<ProfileFile>, ProfileError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(ProfileError::Invalid(format!("couldn't read {FILE_NAME} ({err})"))),
    };
    let raw: Value = serde_json::from_slice(&bytes).map_err(|err| ProfileError::Corrupt(err.to_string()))?;
    let file = upgrade(raw)?;
    file.validate()
        .map_err(|err| ProfileError::Invalid(format!("the profiles file has a setting this version can't use ({err})")))?;
    Ok(Some(file))
}

/// Turns any supported saved format into the current one by running its upgrade steps in order.
fn upgrade(mut raw: Value) -> Result<ProfileFile, ProfileError> {
    let version = raw
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .filter(|&v| v > 0)
        .ok_or_else(|| ProfileError::Invalid("the profiles file has no usable schemaVersion".into()))?;
    if version > SCHEMA_VERSION {
        return Err(ProfileError::TooNew(version));
    }
    for step in UPGRADES.iter().skip(version as usize - 1) {
        raw = step(raw)?;
    }
    if let Some(fields) = raw.as_object_mut() {
        fields.insert("schemaVersion".into(), SCHEMA_VERSION.into());
    }
    serde_json::from_value(raw)
        .map_err(|err| ProfileError::Invalid(format!("the profiles file isn't laid out as expected ({err})")))
}

/// Writes and flushes a temporary file, then swaps it in, so neither a crash nor a power cut
/// mid-save can leave half a file.
fn write_file(path: &Path, file: &ProfileFile) -> Result<(), ProfileError> {
    let io = |err: std::io::Error| ProfileError::Io(err.to_string());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(io)?;
    }
    let json = serde_json::to_string_pretty(file).map_err(|err| ProfileError::Io(err.to_string()))?;
    let temp = path.with_extension("json.tmp");
    let mut out = File::create(&temp).map_err(io)?;
    out.write_all(json.as_bytes()).map_err(io)?;
    out.sync_all().map_err(io)?;
    drop(out);
    fs::rename(&temp, path).map_err(io)
}

/// Moves a damaged file aside and returns the backup's file name.
fn backup_damaged(path: &Path) -> std::io::Result<String> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let name = format!("profiles.damaged-{stamp}.json");
    fs::rename(path, path.with_file_name(&name))?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh folder under the system temp dir for one test, removed when the test ends.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(test: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("keystra-test-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn file(&self) -> PathBuf {
            self.0.join(FILE_NAME)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn first_run_creates_a_default_profile_on_disk() {
        let dir = TempDir::new("first-run");
        let store = ProfileStore::open(dir.file());
        assert_eq!(store.profiles().len(), 1);
        assert_eq!(store.profiles()[0].name, "Default");
        assert!(dir.file().exists());
        assert!(store.list().read_only.is_none());
    }

    #[test]
    fn changes_survive_a_reopen() {
        let dir = TempDir::new("reopen");
        let mut store = ProfileStore::open(dir.file());
        let gaming = store.create("Gaming", Some(1)).unwrap();
        store.rename(gaming.id, "FPS").unwrap();
        let mut edited = store.get(gaming.id).unwrap().clone();
        edited.polling_rate_hz = 8000;
        edited.linked_apps.push("game.exe".into());
        store.update(edited).unwrap();

        let reopened = ProfileStore::open(dir.file());
        let fps = reopened.get(gaming.id).unwrap();
        assert_eq!(fps.name, "FPS");
        assert_eq!(fps.polling_rate_hz, 8000);
        assert_eq!(fps.linked_apps, ["game.exe"]);
    }

    #[test]
    fn a_version_1_file_still_loads() {
        // Written by hand, not by the current code, so a format change that breaks old files fails here.
        let dir = TempDir::new("v1");
        let v1 = r##"{
          "schemaVersion": 1,
          "nextId": 4,
          "profiles": [
            { "id": 1, "name": "Default", "onboardSlot": 0, "pollingRateHz": 2000,
              "lighting": { "mode": 16, "brightness": 4, "speed": 2, "direction": 0, "rainbow": true, "color": "#FFFFFF" } },
            { "id": 3, "name": "Vultify", "onboardSlot": 1, "pollingRateHz": 8000, "linkedApps": ["game.exe"],
              "lighting": { "mode": 1, "brightness": 3, "speed": 1, "direction": 0, "rainbow": false, "color": "#FAFFFA" } }
          ]
        }"##;
        fs::write(dir.file(), v1).unwrap();
        let store = ProfileStore::open(dir.file());
        assert!(store.list().read_only.is_none());
        let names: Vec<_> = store.profiles().iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Default", "Vultify"]);
        assert_eq!(store.get(3).unwrap().linked_apps, ["game.exe"]);
    }

    #[test]
    fn ids_are_never_reused() {
        let dir = TempDir::new("ids");
        let mut store = ProfileStore::open(dir.file());
        let a = store.create("A", None).unwrap();
        store.delete(a.id).unwrap();
        let b = store.create("B", None).unwrap();
        assert!(b.id > a.id);
    }

    #[test]
    fn a_copy_keeps_settings_but_not_linked_programs() {
        let dir = TempDir::new("copy");
        let mut store = ProfileStore::open(dir.file());
        let mut source = store.get(1).unwrap().clone();
        source.polling_rate_hz = 8000;
        source.linked_apps.push("game.exe".into());
        store.update(source).unwrap();
        let copy = store.create("Copy", Some(1)).unwrap();
        assert_eq!(copy.polling_rate_hz, 8000);
        assert!(copy.linked_apps.is_empty());
    }

    #[test]
    fn create_with_saves_the_adjusted_profile_once() {
        let dir = TempDir::new("create-with");
        let mut store = ProfileStore::open(dir.file());
        let created = store.create_with("Read", |p| p.onboard_slot = 2).unwrap();
        assert_eq!(ProfileStore::open(dir.file()).get(created.id).unwrap().onboard_slot, 2);
        assert!(store.create_with("Bad", |p| p.onboard_slot = 9).is_err());
        assert_eq!(store.profiles().len(), 2);
    }

    #[test]
    fn the_last_profile_cannot_be_deleted() {
        let dir = TempDir::new("last");
        let mut store = ProfileStore::open(dir.file());
        assert!(matches!(store.delete(1), Err(ProfileError::LastProfile)));
        assert_eq!(store.profiles().len(), 1);
    }

    #[test]
    fn invalid_changes_are_rejected_and_nothing_is_saved() {
        let dir = TempDir::new("invalid");
        let mut store = ProfileStore::open(dir.file());
        assert!(matches!(store.create("default", None), Err(ProfileError::NameTaken(_))));
        let mut bad = store.get(1).unwrap().clone();
        bad.polling_rate_hz = 3000;
        assert!(store.update(bad).is_err());
        assert!(matches!(store.rename(99, "X"), Err(ProfileError::NotFound)));
        let reopened = ProfileStore::open(dir.file());
        assert_eq!(reopened.profiles(), ProfileFile::fresh().profiles.as_slice());
    }

    #[test]
    fn damaged_file_is_backed_up_and_replaced() {
        let dir = TempDir::new("damaged");
        fs::write(dir.file(), "{ not json").unwrap();
        let store = ProfileStore::open(dir.file());
        assert_eq!(store.profiles().len(), 1);
        assert!(store.list().read_only.is_none());
        let backups = fs::read_dir(&dir.0)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with("profiles.damaged-"))
            .count();
        assert_eq!(backups, 1);
    }

    #[test]
    fn a_file_breaking_a_rule_opens_read_only_and_is_kept() {
        let dir = TempDir::new("rule");
        let saved = r##"{"schemaVersion": 1, "nextId": 2, "profiles": [
          { "id": 1, "name": "Default", "onboardSlot": 0, "pollingRateHz": 3000,
            "lighting": { "mode": 16, "brightness": 4, "speed": 2, "direction": 0, "rainbow": true, "color": "#FFFFFF" } } ]}"##;
        fs::write(dir.file(), saved).unwrap();
        let mut store = ProfileStore::open(dir.file());
        assert!(store.list().read_only.is_some());
        assert!(matches!(store.create("X", None), Err(ProfileError::ReadOnly(_))));
        assert_eq!(fs::read_to_string(dir.file()).unwrap(), saved);
    }

    #[test]
    fn a_wrong_shaped_file_opens_read_only_and_is_kept() {
        let dir = TempDir::new("shape");
        let saved = r#"{"schemaVersion": 1, "nextId": "two", "profiles": []}"#;
        fs::write(dir.file(), saved).unwrap();
        let store = ProfileStore::open(dir.file());
        assert!(store.list().read_only.is_some());
        assert_eq!(fs::read_to_string(dir.file()).unwrap(), saved);
    }

    #[test]
    fn bytes_that_are_not_text_count_as_damage() {
        let dir = TempDir::new("bytes");
        fs::write(dir.file(), [0xFF, 0xFE, 0x00, 0x7B]).unwrap();
        let store = ProfileStore::open(dir.file());
        assert!(store.list().read_only.is_none());
        assert_eq!(store.profiles().len(), 1);
    }

    #[test]
    fn newer_format_is_left_untouched() {
        let dir = TempDir::new("newer");
        let newer = r#"{"schemaVersion": 99, "nextId": 5, "profiles": []}"#;
        fs::write(dir.file(), newer).unwrap();
        let mut store = ProfileStore::open(dir.file());
        assert!(store.list().read_only.is_some());
        assert!(store.create("X", None).is_err(), "read-only");
        assert_eq!(fs::read_to_string(dir.file()).unwrap(), newer);
    }
}
