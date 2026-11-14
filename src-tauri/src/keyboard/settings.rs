//! Turns raw replies into settings the screens can show.

use serde::Serialize;

use super::error::KeyboardError;
use super::protocol::Command;

/// Device IDs reported by GET_USB_VERSION for the MX 8.2 Pro TMR.
const MODELS: [(u32, &str); 2] = [(2494, "MX 8.2 Pro TMR"), (3098, "MX 8.2 Pro TMR (DE)")];

/// Brightness and speed both run from 0 to this value. Speed is stored inverted: the keyboard
/// holds MAX_LEVEL minus the speed.
pub const MAX_LEVEL: u8 = 4;

/// How a lighting mode uses the low nibble of the option byte (byte 4) to pick its color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorRule {
    /// 8 = rainbow, 7 = the stored RGB, 0..6 = a preset color.
    Standard,
    /// Music modes: 0 = rainbow, 4 = the stored RGB.
    Music,
    /// The effect sets its own colors, so the nibble isn't a color choice.
    Unused,
}

struct LightingMode {
    id: u8,
    /// MagCrate's English name, so Diagnostics matches what MagCrate shows.
    name: &'static str,
    color: ColorRule,
}

/// Lighting modes 0 to 22 of the RY5088 firmware. Mode 20 isn't offered on the MX 8.2 but is named
/// in case the keyboard ever reports it; modes 23 to 25 belong to other keyboards and read as unknown.
const LIGHTING_MODES: [LightingMode; 23] = [
    LightingMode { id: 0, name: "Off", color: ColorRule::Unused },
    LightingMode { id: 1, name: "Constant Light", color: ColorRule::Standard },
    LightingMode { id: 2, name: "Breathing", color: ColorRule::Standard },
    LightingMode { id: 3, name: "Spectrum Cycle", color: ColorRule::Unused },
    LightingMode { id: 4, name: "Flowing Light", color: ColorRule::Standard },
    LightingMode { id: 5, name: "Water Ripple", color: ColorRule::Standard },
    LightingMode { id: 6, name: "Star Dots", color: ColorRule::Standard },
    LightingMode { id: 7, name: "Flow", color: ColorRule::Standard },
    LightingMode { id: 8, name: "Shadow", color: ColorRule::Standard },
    LightingMode { id: 9, name: "Layers", color: ColorRule::Standard },
    LightingMode { id: 10, name: "Sine Wave", color: ColorRule::Standard },
    LightingMode { id: 11, name: "Color Spring", color: ColorRule::Standard },
    LightingMode { id: 12, name: "Neon", color: ColorRule::Standard },
    LightingMode { id: 13, name: "Custom", color: ColorRule::Unused },
    LightingMode { id: 14, name: "Radiant", color: ColorRule::Standard },
    LightingMode { id: 15, name: "Loop", color: ColorRule::Standard },
    LightingMode { id: 16, name: "Color Grid", color: ColorRule::Standard },
    LightingMode { id: 17, name: "Snowfall", color: ColorRule::Standard },
    LightingMode { id: 18, name: "Meteor", color: ColorRule::Standard },
    LightingMode { id: 19, name: "Silent Snow", color: ColorRule::Standard },
    LightingMode { id: 20, name: "Music Sync 3", color: ColorRule::Music },
    LightingMode { id: 21, name: "Light Shadow", color: ColorRule::Unused },
    LightingMode { id: 22, name: "Music Sync", color: ColorRule::Music },
];

const STANDARD_RAINBOW: u8 = 8;
const STANDARD_SINGLE_COLOR: u8 = 7;
const MUSIC_RAINBOW: u8 = 0;

/// Preset colors the option byte can point to instead of the RGB bytes.
const PRESET_COLORS: [[u8; 3]; 7] = [
    [0xFF, 0x00, 0x00],
    [0xFF, 0x80, 0x00],
    [0xFF, 0xFF, 0x00],
    [0x00, 0xFF, 0x00],
    [0x00, 0xFF, 0xFF],
    [0x00, 0x00, 0xFF],
    [0xFF, 0x00, 0xFF],
];

/// The firmware stores white as FA FF FA.
const STORED_WHITE: [u8; 3] = [0xFA, 0xFF, 0xFA];

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub device_id: u32,
    pub firmware_version: u16,
    pub model: Option<&'static str>,
}

impl DeviceInfo {
    pub fn is_supported(&self) -> bool {
        self.model.is_some()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Lighting {
    pub mode_id: u8,
    pub mode_name: Option<&'static str>,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub rainbow: bool,
    /// Hex color like "#FFFFFF", or `None` when the effect picks its own colors.
    /// Ignored by the keyboard when `rainbow` is on.
    pub color: Option<String>,
}

fn field(command: Command, reply: &[u8], index: usize) -> Result<u8, KeyboardError> {
    reply.get(index).copied().ok_or_else(|| KeyboardError::BadReply {
        command: command.name(),
        reason: format!("reply is {} bytes, needed byte {index}", reply.len()),
    })
}

pub fn parse_device_info(reply: &[u8]) -> Result<DeviceInfo, KeyboardError> {
    let command = Command::GetDeviceInfo;
    let id_bytes = [
        field(command, reply, 1)?,
        field(command, reply, 2)?,
        field(command, reply, 3)?,
        field(command, reply, 4)?,
    ];
    let device_id = u32::from_le_bytes(id_bytes);
    let firmware_version = u16::from_le_bytes([field(command, reply, 7)?, field(command, reply, 8)?]);
    let model = MODELS.iter().find(|(id, _)| *id == device_id).map(|(_, name)| *name);
    Ok(DeviceInfo { device_id, firmware_version, model })
}

/// Returns the active onboard profile slot, counting from 0.
pub fn parse_onboard_slot(reply: &[u8]) -> Result<u8, KeyboardError> {
    field(Command::GetProfile, reply, 1)
}

/// Polling rates in Hz by firmware code: the code is the index. Shared by reads and (Phase 4) writes.
pub const POLLING_RATES: [u32; 7] = [8000, 4000, 2000, 1000, 500, 250, 125];

/// Returns the polling rate in Hz, or `None` for a code the firmware doesn't document.
pub fn parse_polling_rate(reply: &[u8]) -> Result<Option<u32>, KeyboardError> {
    let code = field(Command::GetPollingRate, reply, 2)?;
    Ok(POLLING_RATES.get(usize::from(code)).copied())
}

/// True for a lighting mode ID this firmware family names.
pub fn is_known_lighting_mode(mode_id: u8) -> bool {
    LIGHTING_MODES.iter().any(|m| m.id == mode_id)
}

/// Every named lighting mode as (id, MagCrate name, whether it uses the chosen color), for the screens.
#[cfg(test)]
pub fn lighting_modes() -> impl Iterator<Item = (u8, &'static str, bool)> {
    LIGHTING_MODES.iter().map(|m| (m.id, m.name, m.color != ColorRule::Unused))
}

pub fn parse_lighting(reply: &[u8]) -> Result<Lighting, KeyboardError> {
    let command = Command::GetLighting;
    let mode_id = field(command, reply, 1)?;
    let speed = MAX_LEVEL.saturating_sub(field(command, reply, 2)?);
    let brightness = field(command, reply, 3)?;
    let option = field(command, reply, 4)?;
    let stored = [field(command, reply, 5)?, field(command, reply, 6)?, field(command, reply, 7)?];

    let mode = LIGHTING_MODES.iter().find(|m| m.id == mode_id);
    let flag = option & 0x0F;
    let (rainbow, rgb) = match mode.map_or(ColorRule::Standard, |m| m.color) {
        ColorRule::Unused => (false, None),
        ColorRule::Music => (flag == MUSIC_RAINBOW, Some(stored)),
        ColorRule::Standard => match flag {
            STANDARD_RAINBOW => (true, Some(stored)),
            STANDARD_SINGLE_COLOR => (false, Some(stored)),
            preset => (false, Some(PRESET_COLORS.get(usize::from(preset)).copied().unwrap_or(stored))),
        },
    };

    Ok(Lighting {
        mode_id,
        mode_name: mode.map(|m| m.name),
        brightness,
        speed,
        direction: option >> 4,
        rainbow,
        color: rgb.map(to_hex),
    })
}

fn to_hex(rgb: [u8; 3]) -> String {
    let [r, g, b] = if rgb == STORED_WHITE { [0xFF; 3] } else { rgb };
    format!("#{r:02X}{g:02X}{b:02X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(bytes: &[u8]) -> Vec<u8> {
        let mut full = bytes.to_vec();
        full.resize(64, 0);
        full
    }

    #[test]
    fn device_info_reads_little_endian_id_and_version() {
        // 2494 = 0x09BE, firmware 0x0104.
        let info = parse_device_info(&reply(&[0x8F, 0xBE, 0x09, 0, 0, 0, 0, 0x04, 0x01])).unwrap();
        assert_eq!(info.device_id, 2494);
        assert_eq!(info.firmware_version, 0x0104);
        assert_eq!(info.model, Some("MX 8.2 Pro TMR"));
        assert!(info.is_supported());
    }

    #[test]
    fn unknown_device_is_not_supported() {
        let info = parse_device_info(&reply(&[0x8F, 0x01, 0, 0, 0])).unwrap();
        assert!(!info.is_supported());
    }

    #[test]
    fn short_reply_is_an_error_not_a_panic() {
        assert!(parse_device_info(&[0x8F, 0xBE]).is_err());
        assert!(parse_lighting(&[0x87]).is_err());
    }

    #[test]
    fn polling_codes_map_to_hz() {
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 2])).unwrap(), Some(2000));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 0])).unwrap(), Some(8000));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 6])).unwrap(), Some(125));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 9])).unwrap(), None);
    }

    #[test]
    fn profile_is_byte_one() {
        assert_eq!(parse_onboard_slot(&reply(&[0x84, 2])).unwrap(), 2);
    }

    #[test]
    fn lighting_single_color() {
        // Water Ripple, speed byte 1 (speed 3), brightness 4, direction 0, single color, red.
        let lighting = parse_lighting(&reply(&[0x87, 5, 1, 4, 0x07, 0xFF, 0x00, 0x00])).unwrap();
        assert_eq!(lighting.mode_name, Some("Water Ripple"));
        assert_eq!(lighting.speed, 3);
        assert_eq!(lighting.brightness, 4);
        assert!(!lighting.rainbow);
        assert_eq!(lighting.color.as_deref(), Some("#FF0000"));
    }

    #[test]
    fn lighting_rainbow_and_direction() {
        let lighting = parse_lighting(&reply(&[0x87, 4, 0, 2, 0x18, 0, 0, 0])).unwrap();
        assert!(lighting.rainbow);
        assert_eq!(lighting.direction, 1);
        assert_eq!(lighting.speed, 4);
    }

    #[test]
    fn real_keyboard_reply_reads_as_color_grid_rainbow() {
        // Captured from the MX 8.2: 87 10 02 04 08 FA FF FA.
        let lighting = parse_lighting(&reply(&[0x87, 0x10, 0x02, 0x04, 0x08, 0xFA, 0xFF, 0xFA])).unwrap();
        assert_eq!(lighting.mode_name, Some("Color Grid"));
        assert!(lighting.rainbow);
        assert_eq!(lighting.speed, 2);
    }

    #[test]
    fn lighting_preset_and_stored_white() {
        let preset = parse_lighting(&reply(&[0x87, 1, 0, 4, 0x05, 0, 0, 0])).unwrap();
        assert_eq!(preset.color.as_deref(), Some("#0000FF"));
        let white = parse_lighting(&reply(&[0x87, 1, 0, 4, 0x07, 0xFA, 0xFF, 0xFA])).unwrap();
        assert_eq!(white.color.as_deref(), Some("#FFFFFF"));
    }

    #[test]
    fn music_sync_uses_its_own_color_flags() {
        // Music Sync: low nibble 0 is rainbow, 4 is the stored color (not preset 4, cyan).
        let rainbow = parse_lighting(&reply(&[0x87, 22, 0, 4, 0x10, 0x12, 0x34, 0x56])).unwrap();
        assert_eq!(rainbow.mode_name, Some("Music Sync"));
        assert!(rainbow.rainbow);
        let single = parse_lighting(&reply(&[0x87, 22, 0, 4, 0x14, 0x12, 0x34, 0x56])).unwrap();
        assert!(!single.rainbow);
        assert_eq!(single.color.as_deref(), Some("#123456"));
        assert_eq!(single.direction, 1);
    }

    #[test]
    fn effects_with_their_own_colors_report_no_color() {
        for mode in [0, 3, 13, 21] {
            let lighting = parse_lighting(&reply(&[0x87, mode, 0, 4, 0x00, 0xFF, 0, 0])).unwrap();
            assert_eq!(lighting.color, None, "mode {mode}");
            assert!(!lighting.rainbow, "mode {mode}");
        }
    }

    #[test]
    fn mode_names_match_magcrate() {
        let name = |id| LIGHTING_MODES.iter().find(|m| m.id == id).map(|m| m.name);
        assert_eq!(name(3), Some("Spectrum Cycle"));
        assert_eq!(name(12), Some("Neon"));
        assert_eq!(name(21), Some("Light Shadow"));
        let ids: Vec<u8> = LIGHTING_MODES.iter().map(|m| m.id).collect();
        assert_eq!(ids, (0..=22).collect::<Vec<u8>>(), "one entry per mode, in order");
    }

    #[test]
    fn unknown_mode_falls_back_to_the_standard_rule() {
        let lighting = parse_lighting(&reply(&[0x87, 40, 0, 4, 0x08, 0, 0, 0])).unwrap();
        assert_eq!(lighting.mode_name, None);
        assert!(lighting.rainbow);
    }

    #[test]
    fn speed_byte_above_max_does_not_underflow() {
        let lighting = parse_lighting(&reply(&[0x87, 1, 9, 4, 0x07, 0, 0, 0])).unwrap();
        assert_eq!(lighting.speed, 0);
    }
}
