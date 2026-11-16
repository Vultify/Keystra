//! Lighting in both directions: reading the keyboard's lighting reply and building the bytes for a
//! lighting write, both from one table of effects taken from MagCrate's MX 8.2 layout.

use serde::{Deserialize, Serialize};

use super::error::KeyboardError;
use super::protocol::Command;
use super::settings::field;

/// Brightness and speed both run from 0 to this value. Speed is stored inverted: the keyboard
/// holds MAX_LEVEL minus the speed.
pub const MAX_LEVEL: u8 = 4;

/// How an effect uses the low nibble of the option byte (byte 4) to pick its color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorRule {
    /// 8 = rainbow, 7 = the stored RGB, 0..6 = a preset color.
    Standard,
    /// Music effects: 0 = rainbow, 4 = the stored RGB.
    Music,
    /// The effect picks its own colors.
    Own,
    /// Custom: the high nibble picks one of the 5 saved per-key patterns.
    Pattern,
    /// Light Shadow: colors come from the screen, so the option byte is always 0.
    Screen,
}

impl ColorRule {
    /// Whether the effect shows a chosen color or rainbow. Exhaustive on purpose, so a new rule
    /// can't be left out.
    const fn uses_color(self) -> bool {
        match self {
            ColorRule::Standard | ColorRule::Music => true,
            ColorRule::Own | ColorRule::Pattern | ColorRule::Screen => false,
        }
    }
}

struct LightingMode {
    id: u8,
    /// MagCrate's English name, so Keystra matches what MagCrate shows.
    name: &'static str,
    color: ColorRule,
    brightness: bool,
    speed: bool,
    /// The choices the high nibble of the option byte selects, in order (direction, pattern).
    options: &'static [&'static str],
    /// False for effects the MX 8.2 doesn't offer. Keystra reads them but never writes them.
    offered: bool,
}

#[cfg(test)]
impl LightingMode {
    /// What the effect's option is called on screen, or `None` when it has no options.
    const fn option_label(&self) -> Option<&'static str> {
        match self.color {
            _ if self.options.is_empty() => None,
            ColorRule::Pattern => Some("Pattern"),
            ColorRule::Music => Some("Style"),
            _ => Some("Direction"),
        }
    }
}

const fn mode(id: u8, name: &'static str, color: ColorRule, speed: bool, options: &'static [&'static str]) -> LightingMode {
    LightingMode { id, name, color, brightness: true, speed, options, offered: true }
}

const NONE: &[&str] = &[];

/// Effects 0 to 22 of the RY5088 firmware, with the MX 8.2's options from MagCrate. Effects 23 to
/// 25 belong to other keyboards and read as unknown.
const LIGHTING_MODES: [LightingMode; 23] = [
    LightingMode { id: 0, name: "Off", color: ColorRule::Own, brightness: false, speed: false, options: NONE, offered: true },
    mode(1, "Constant Light", ColorRule::Standard, false, NONE),
    mode(2, "Breathing", ColorRule::Standard, true, NONE),
    mode(3, "Spectrum Cycle", ColorRule::Own, true, NONE),
    mode(4, "Flowing Light", ColorRule::Standard, true, &["Right", "Left", "Down", "Up"]),
    mode(5, "Water Ripple", ColorRule::Standard, true, NONE),
    mode(6, "Star Dots", ColorRule::Standard, true, NONE),
    mode(7, "Flow", ColorRule::Standard, true, &["Zigzag", "Spiral"]),
    mode(8, "Shadow", ColorRule::Standard, true, NONE),
    mode(9, "Layers", ColorRule::Standard, true, NONE),
    mode(10, "Sine Wave", ColorRule::Standard, true, NONE),
    mode(11, "Color Spring", ColorRule::Standard, true, &["Outward", "Inward"]),
    mode(12, "Neon", ColorRule::Standard, true, &["Right", "Left"]),
    mode(13, "Custom", ColorRule::Pattern, false, &["Pattern 1", "Pattern 2", "Pattern 3", "Pattern 4", "Pattern 5"]),
    mode(14, "Radiant", ColorRule::Standard, true, NONE),
    mode(15, "Loop", ColorRule::Standard, true, &["Counterclockwise", "Clockwise"]),
    mode(16, "Color Grid", ColorRule::Standard, true, NONE),
    mode(17, "Snowfall", ColorRule::Standard, true, NONE),
    mode(18, "Meteor", ColorRule::Standard, true, NONE),
    mode(19, "Silent Snow", ColorRule::Standard, true, NONE),
    LightingMode { id: 20, name: "Music Sync 3", color: ColorRule::Music, brightness: true, speed: false, options: NONE, offered: false },
    mode(21, "Light Shadow", ColorRule::Screen, false, NONE),
    mode(22, "Music Sync", ColorRule::Music, false, &["Upright", "Separate", "Intersect"]),
];

const STANDARD_RAINBOW: u8 = 8;
const STANDARD_SINGLE_COLOR: u8 = 7;
const MUSIC_RAINBOW: u8 = 0;
const MUSIC_SINGLE_COLOR: u8 = 4;

/// What MagCrate sends as the color for Custom, which ignores it.
const PATTERN_RGB: [u8; 3] = [0, 200, 200];

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

fn find_mode(id: u8) -> Option<&'static LightingMode> {
    LIGHTING_MODES.iter().find(|m| m.id == id)
}

/// What the keyboard reported, ready to show.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Lighting {
    pub mode_id: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub rainbow: bool,
    /// Hex color like "#FFFFFF", or `None` when the effect picks its own colors.
    /// Ignored by the keyboard when `rainbow` is on.
    pub color: Option<String>,
}

impl Lighting {
    /// True when the keyboard holds `wanted`, comparing only what the effect actually uses.
    pub fn matches(&self, wanted: &LightingSettings) -> bool {
        let Some(mode) = find_mode(wanted.mode) else { return false };
        let uses_color = mode.color.uses_color();
        self.mode_id == wanted.mode
            && (!mode.brightness || self.brightness == wanted.brightness)
            && (!mode.speed || self.speed == wanted.speed)
            && (mode.options.is_empty() || self.direction == wanted.direction)
            && (!uses_color || self.rainbow == wanted.rainbow)
            && (!uses_color
                || wanted.rainbow
                || self.color.as_deref().and_then(stored_rgb) == stored_rgb(&wanted.color))
    }
}

/// The lighting values the firmware takes, in the form Keystra saves and writes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LightingSettings {
    pub mode: u8,
    pub brightness: u8,
    pub speed: u8,
    /// The effect's option: its direction, or the pattern for Custom.
    pub direction: u8,
    pub rainbow: bool,
    /// "#RRGGBB". Kept even for effects that pick their own colors, so it survives a mode change.
    pub color: String,
}

impl LightingSettings {
    /// The MX 8.2's out-of-the-box lighting: Color Grid, rainbow, full brightness, speed 2.
    pub fn factory() -> Self {
        Self { mode: 16, brightness: MAX_LEVEL, speed: 2, direction: 0, rainbow: true, color: "#FFFFFF".into() }
    }

    /// Takes what the keyboard reported. Effects without a color keep `fallback_color`.
    pub fn from_reading(reading: &Lighting, fallback_color: &str) -> Self {
        Self {
            mode: reading.mode_id,
            brightness: reading.brightness,
            speed: reading.speed,
            direction: reading.direction,
            rainbow: reading.rainbow,
            color: reading.color.clone().unwrap_or_else(|| fallback_color.to_string()),
        }
    }

    /// Checks the values are ones the firmware can hold. Writes check more (see `encode`).
    pub fn validate(&self) -> Result<(), String> {
        if find_mode(self.mode).is_none() {
            return Err("unknown lighting effect".into());
        }
        if self.brightness > MAX_LEVEL || self.speed > MAX_LEVEL {
            return Err(format!("brightness and speed must be 0 to {MAX_LEVEL}"));
        }
        if self.direction > 0x0F {
            return Err("direction is out of range".into());
        }
        parse_hex(&self.color).map(|_| ()).ok_or_else(|| "color must look like #RRGGBB".into())
    }

    /// Builds bytes 1 to 7 of a lighting write, the way MagCrate does. Refuses effects the MX 8.2
    /// doesn't offer and options the effect doesn't have.
    pub fn encode(&self) -> Result<[u8; 7], KeyboardError> {
        let invalid = |reason: String| KeyboardError::InvalidSetting(reason);
        self.validate().map_err(invalid)?;
        let mode = find_mode(self.mode).filter(|m| m.offered).ok_or_else(|| {
            invalid(format!("effect {} isn't available on this keyboard", self.mode))
        })?;
        if !mode.options.is_empty() && usize::from(self.direction) >= mode.options.len() {
            return Err(invalid(format!("{} has no option {}", mode.name, self.direction)));
        }
        let rgb = stored_rgb(&self.color).expect("checked by validate");
        let high = self.direction << 4;
        let (option, rgb) = match mode.color {
            ColorRule::Standard => (high | if self.rainbow { STANDARD_RAINBOW } else { STANDARD_SINGLE_COLOR }, rgb),
            ColorRule::Music => (high | if self.rainbow { MUSIC_RAINBOW } else { MUSIC_SINGLE_COLOR }, rgb),
            ColorRule::Own => (high | STANDARD_SINGLE_COLOR, rgb),
            ColorRule::Pattern => (high, PATTERN_RGB),
            ColorRule::Screen => (0, rgb),
        };
        let [r, g, b] = rgb;
        Ok([self.mode, MAX_LEVEL - self.speed, self.brightness, option, r, g, b])
    }
}

/// One effect as the screens get it, in the generated constants.
#[cfg(test)]
pub struct ModeInfo {
    pub id: u8,
    pub name: &'static str,
    pub option_label: Option<&'static str>,
    pub uses_color: bool,
    pub brightness: bool,
    pub speed: bool,
    pub options: &'static [&'static str],
    pub offered: bool,
}

#[cfg(test)]
pub fn lighting_modes() -> impl Iterator<Item = ModeInfo> {
    LIGHTING_MODES.iter().map(|m| ModeInfo {
        id: m.id,
        name: m.name,
        option_label: m.option_label(),
        uses_color: m.color.uses_color(),
        brightness: m.brightness,
        speed: m.speed,
        options: m.options,
        offered: m.offered,
    })
}

pub fn parse_lighting(reply: &[u8]) -> Result<Lighting, KeyboardError> {
    let command = Command::GetLighting;
    let mode_id = field(command, reply, 1)?;
    let speed = MAX_LEVEL.saturating_sub(field(command, reply, 2)?);
    let brightness = field(command, reply, 3)?;
    let option = field(command, reply, 4)?;
    let stored = [field(command, reply, 5)?, field(command, reply, 6)?, field(command, reply, 7)?];

    let mode = find_mode(mode_id);
    let flag = option & 0x0F;
    let (rainbow, rgb) = match mode.map_or(ColorRule::Standard, |m| m.color) {
        rule if !rule.uses_color() => (false, None),
        ColorRule::Music => (flag == MUSIC_RAINBOW, Some(stored)),
        _ => match flag {
            STANDARD_RAINBOW => (true, Some(stored)),
            STANDARD_SINGLE_COLOR => (false, Some(stored)),
            preset => (false, Some(PRESET_COLORS.get(usize::from(preset)).copied().unwrap_or(stored))),
        },
    };

    Ok(Lighting {
        mode_id,
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

/// The RGB bytes the firmware holds for a color: white is stored as FA FF FA.
fn stored_rgb(color: &str) -> Option<[u8; 3]> {
    parse_hex(color).map(|rgb| if rgb == [0xFF; 3] { STORED_WHITE } else { rgb })
}

pub(super) fn parse_hex(color: &str) -> Option<[u8; 3]> {
    let digits = color.strip_prefix('#').filter(|d| d.len() == 6 && d.chars().all(|c| c.is_ascii_hexdigit()))?;
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(bytes: &[u8]) -> Vec<u8> {
        let mut full = bytes.to_vec();
        full.resize(64, 0);
        full
    }

    fn settings(mode: u8, rainbow: bool, color: &str) -> LightingSettings {
        LightingSettings { mode, brightness: 3, speed: 1, direction: 0, rainbow, color: color.into() }
    }

    #[test]
    fn lighting_single_color() {
        // Water Ripple, speed byte 1 (speed 3), brightness 4, direction 0, single color, red.
        let lighting = parse_lighting(&reply(&[0x87, 5, 1, 4, 0x07, 0xFF, 0x00, 0x00])).unwrap();
        assert_eq!(lighting.mode_id, 5);
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
        assert_eq!(lighting.mode_id, 16);
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
        assert_eq!(rainbow.mode_id, 22);
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
        let name = |id| find_mode(id).map(|m| m.name);
        assert_eq!(name(3), Some("Spectrum Cycle"));
        assert_eq!(name(12), Some("Neon"));
        assert_eq!(name(21), Some("Light Shadow"));
        let ids: Vec<u8> = LIGHTING_MODES.iter().map(|m| m.id).collect();
        assert_eq!(ids, (0..=22).collect::<Vec<u8>>(), "one entry per mode, in order");
    }

    #[test]
    fn unknown_mode_falls_back_to_the_standard_rule() {
        let lighting = parse_lighting(&reply(&[0x87, 40, 0, 4, 0x08, 0, 0, 0])).unwrap();
        assert!(find_mode(lighting.mode_id).is_none());
        assert!(lighting.rainbow);
    }

    #[test]
    fn speed_byte_above_max_does_not_underflow() {
        let lighting = parse_lighting(&reply(&[0x87, 1, 9, 4, 0x07, 0, 0, 0])).unwrap();
        assert_eq!(lighting.speed, 0);
    }

    #[test]
    fn encodes_like_magcrate() {
        // Color Grid rainbow at speed 2, brightness 4, white: the reply captured from the keyboard.
        let factory = LightingSettings::factory();
        assert_eq!(factory.encode().unwrap(), [0x10, 0x02, 0x04, 0x08, 0xFA, 0xFF, 0xFA]);
        // Water Ripple in solid red, speed 1, brightness 3.
        assert_eq!(settings(5, false, "#FF0000").encode().unwrap(), [5, 3, 3, 0x07, 0xFF, 0, 0]);
        // Music Sync uses 0 for rainbow and 4 for the stored color.
        assert_eq!(settings(22, true, "#123456").encode().unwrap()[3], 0x00);
        assert_eq!(settings(22, false, "#123456").encode().unwrap()[3], 0x04);
        // Custom puts the pattern in the high nibble and sends MagCrate's placeholder color.
        let mut custom = settings(13, false, "#FF0000");
        custom.direction = 2;
        assert_eq!(custom.encode().unwrap()[3..], [0x20, 0, 200, 200]);
        // Light Shadow always sends a zero option byte.
        assert_eq!(settings(21, true, "#FF0000").encode().unwrap()[3], 0);
    }

    #[test]
    fn every_offered_effect_reads_back_as_what_was_written() {
        for mode in LIGHTING_MODES.iter().filter(|m| m.offered) {
            for rainbow in [false, true] {
                let mut wanted = settings(mode.id, rainbow, "#20C0FF");
                wanted.direction = u8::try_from(mode.options.len().saturating_sub(1)).unwrap();
                let bytes = wanted.encode().unwrap();
                let mut raw = vec![0x87];
                raw.extend_from_slice(&bytes);
                let read = parse_lighting(&reply(&raw)).unwrap();
                assert!(read.matches(&wanted), "{} rainbow={rainbow}: {read:?}", mode.name);
            }
        }
    }

    #[test]
    fn stored_white_matches_both_spellings() {
        // The firmware keeps white as FA FF FA, so #FAFFFA and #FFFFFF read back the same.
        for color in ["#FFFFFF", "#FAFFFA", "#faffFA"] {
            let wanted = settings(1, false, color);
            let mut raw = vec![0x87];
            raw.extend_from_slice(&wanted.encode().unwrap());
            assert!(parse_lighting(&reply(&raw)).unwrap().matches(&wanted), "{color}");
        }
    }

    #[test]
    fn option_labels_follow_the_effect() {
        let label = |id| find_mode(id).and_then(|m| m.option_label());
        assert_eq!(label(4), Some("Direction"));
        assert_eq!(label(13), Some("Pattern"));
        assert_eq!(label(22), Some("Style"));
        assert_eq!(label(16), None);
    }

    #[test]
    fn writes_refuse_what_the_keyboard_does_not_offer() {
        assert!(settings(20, false, "#FFFFFF").encode().is_err(), "Music Sync 3");
        let mut neon = settings(12, false, "#FFFFFF");
        neon.direction = 2;
        assert!(neon.encode().is_err(), "Neon has two directions");
        assert!(settings(40, false, "#FFFFFF").encode().is_err());
        assert!(settings(1, false, "white").encode().is_err());
    }

    #[test]
    fn a_different_reading_does_not_match() {
        let wanted = settings(5, false, "#FF0000");
        let read = parse_lighting(&reply(&[0x87, 5, 3, 3, 0x07, 0x00, 0xFF, 0x00])).unwrap();
        assert!(!read.matches(&wanted), "color differs");
        let read = parse_lighting(&reply(&[0x87, 6, 3, 3, 0x07, 0xFF, 0x00, 0x00])).unwrap();
        assert!(!read.matches(&wanted), "effect differs");
    }
}
