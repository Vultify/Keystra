//! What the profiles file holds. Lighting and polling rate are shared by all of the keyboard's
//! onboard profiles, so Keystra keeps them per profile and sends them on every switch.

use serde::{Deserialize, Serialize};

use super::error::ProfileError;
use crate::keyboard::{self, LightingSettings, ONBOARD_SLOTS, POLLING_RATES};

/// Bump when the saved format changes. New fields use `#[serde(default)]`, and each bump adds a
/// step to `store::UPGRADES` (the build fails until it does).
pub const SCHEMA_VERSION: u32 = 2;

pub const MAX_NAME_LEN: usize = 40;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: u32,
    pub name: String,
    /// Which onboard profile (below ONBOARD_SLOTS) holds this profile's key mapping and actuation.
    pub onboard_slot: u8,
    pub lighting: LightingSettings,
    pub polling_rate_hz: u32,
    /// Program names (like "game.exe") that switch to this profile when they're in front (Phase 7).
    #[serde(default)]
    pub linked_apps: Vec<String>,
}

/// The whole saved file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileFile {
    pub schema_version: u32,
    pub next_id: u32,
    pub profiles: Vec<Profile>,
    /// The profile last applied to the keyboard (added in format 2).
    #[serde(default)]
    pub active_id: Option<u32>,
}

/// The profile list as the screens get it, from `list_profiles` and with every PROFILES_EVENT.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct ProfileList {
    pub profiles: Vec<Profile>,
    /// The profile last applied to the keyboard, if any.
    pub active_id: Option<u32>,
    /// Why changes are switched off, when the saved file couldn't be used. That file is left as it was.
    pub read_only: Option<String>,
}

impl Profile {
    pub fn factory(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.into(),
            onboard_slot: 0,
            lighting: LightingSettings::factory(),
            polling_rate_hz: 2000,
            linked_apps: Vec::new(),
        }
    }

    /// Copies what the keyboard reported: its lighting, polling rate and active onboard profile.
    pub fn take_reading(&mut self, snapshot: &keyboard::Snapshot) {
        self.lighting = LightingSettings::from_reading(&snapshot.lighting, &self.lighting.color);
        self.onboard_slot = snapshot.onboard_slot;
        if let Some(hz) = snapshot.polling_rate {
            self.polling_rate_hz = hz;
        }
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        validate_name(&self.name)?;
        if self.onboard_slot >= ONBOARD_SLOTS {
            return Err(ProfileError::Invalid(format!("onboard profile must be 1 to {ONBOARD_SLOTS}")));
        }
        if !POLLING_RATES.contains(&self.polling_rate_hz) {
            return Err(ProfileError::Invalid("polling rate isn't one the keyboard supports".into()));
        }
        self.lighting.validate().map_err(ProfileError::Invalid)
    }
}

impl ProfileFile {
    /// A first-run file with one profile using the keyboard's factory settings.
    pub fn fresh() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            next_id: 2,
            profiles: vec![Profile::factory(1, "Default")],
            active_id: None,
        }
    }

    /// Checks every profile plus the rules that span the whole file.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.profiles.is_empty() {
            return Err(ProfileError::Invalid("there must be at least one profile".into()));
        }
        for (i, profile) in self.profiles.iter().enumerate() {
            profile.validate()?;
            let rest = &self.profiles[i + 1..];
            if rest.iter().any(|p| p.id == profile.id) {
                return Err(ProfileError::Invalid(format!("profile id {} is used twice", profile.id)));
            }
            if rest.iter().any(|p| same_name(&p.name, &profile.name)) {
                return Err(ProfileError::NameTaken(profile.name.clone()));
            }
            if profile.id >= self.next_id {
                return Err(ProfileError::Invalid("next id is behind the profiles".into()));
            }
        }
        if let Some(id) = self.active_id
            && !self.profiles.iter().any(|p| p.id == id)
        {
            return Err(ProfileError::Invalid("the active profile doesn't exist".into()));
        }
        Ok(())
    }
}

/// Names are trimmed, 1 to MAX_NAME_LEN characters.
pub fn validate_name(name: &str) -> Result<(), ProfileError> {
    let length = name.trim().chars().count();
    if length == 0 || length > MAX_NAME_LEN || name.trim() != name {
        return Err(ProfileError::Invalid(format!(
            "name must be 1 to {MAX_NAME_LEN} characters, without spaces at the ends"
        )));
    }
    Ok(())
}

pub fn same_name(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_file_is_valid() {
        ProfileFile::fresh().validate().unwrap();
    }

    #[test]
    fn rejects_out_of_range_settings() {
        let base = Profile::factory(1, "Test");
        let mut p = base.clone();
        p.onboard_slot = 4;
        assert!(p.validate().is_err());
        let mut p = base.clone();
        p.polling_rate_hz = 3000;
        assert!(p.validate().is_err());
        let mut p = base.clone();
        p.lighting.brightness = 5;
        assert!(p.validate().is_err());
        let mut p = base.clone();
        p.lighting.mode = 40;
        assert!(p.validate().is_err());
        let mut p = base;
        p.lighting.color = "white".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn names_are_trimmed_and_bounded() {
        assert!(validate_name("Gaming").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
        assert!(validate_name(" padded ").is_err());
        assert!(validate_name(&"x".repeat(MAX_NAME_LEN + 1)).is_err());
        assert!(same_name("Gaming", " gaming"));
    }

    #[test]
    fn file_rejects_duplicate_ids_and_names() {
        let mut file = ProfileFile::fresh();
        file.profiles.push(Profile::factory(1, "Other"));
        file.next_id = 3;
        assert!(file.validate().is_err(), "duplicate id");

        let mut file = ProfileFile::fresh();
        file.profiles.push(Profile::factory(2, "default"));
        file.next_id = 3;
        assert!(matches!(file.validate(), Err(ProfileError::NameTaken(_))));
    }

    #[test]
    fn missing_linked_apps_loads_as_empty() {
        // Older or hand-edited files may leave optional fields out.
        let json = r##"{"id":1,"name":"Default","onboardSlot":0,"pollingRateHz":2000,
            "lighting":{"mode":16,"brightness":4,"speed":2,"direction":0,"rainbow":true,"color":"#FFFFFF"}}"##;
        let profile: Profile = serde_json::from_str(json).unwrap();
        assert!(profile.linked_apps.is_empty());
    }

    #[test]
    fn reading_without_a_color_keeps_the_saved_one() {
        let reading = keyboard::Lighting {
            mode_id: 21,
            brightness: 3,
            speed: 1,
            direction: 0,
            rainbow: false,
            color: None,
        };
        let settings = LightingSettings::from_reading(&reading, "#123456");
        assert_eq!(settings.color, "#123456");
        assert_eq!(settings.mode, 21);
    }
}
