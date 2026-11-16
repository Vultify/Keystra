//! Custom per-key color patterns, the 5 the keyboard keeps for its Custom effect (layout in
//! research/PROTOCOL-NOTES.md). Reads and writes work on any `Transport`, so they're tested without USB.

use serde::{Deserialize, Serialize};

use super::error::KeyboardError;
use super::lighting::parse_hex;
use super::protocol::{
    Command, PATTERN_BYTES, PATTERN_CHUNK_LEN, PATTERN_READ_PAGES, PATTERN_WRITE_PACKETS, PatternChunk, check_pattern_slot,
};
use super::transport::{self, Transport};

/// Lights a pattern covers, by light number (column x 6 + row).
pub const LIGHT_COUNT: usize = PATTERN_BYTES / 3;

/// How many times a read is repeated when its two copies disagree. Pattern pages carry no echo,
/// so two matching reads are how Keystra knows no other app's reply got mixed in.
const READ_ATTEMPTS: usize = 3;

/// One pattern: a "#RRGGBB" color for every light, in light-number order. Lights with no key are black.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Pattern(pub Vec<String>);

impl Pattern {
    fn from_bytes(bytes: &[u8]) -> Self {
        Pattern(bytes.as_chunks::<3>().0.iter().take(LIGHT_COUNT).map(|[r, g, b]| format!("#{r:02X}{g:02X}{b:02X}")).collect())
    }

    /// The bytes the keyboard stores, after checking every color first.
    fn to_bytes(&self) -> Result<[u8; PATTERN_BYTES], KeyboardError> {
        if self.0.len() != LIGHT_COUNT {
            return Err(KeyboardError::InvalidSetting(format!("a pattern has {LIGHT_COUNT} lights, not {}", self.0.len())));
        }
        let mut bytes = [0u8; PATTERN_BYTES];
        for (light, color) in self.0.iter().enumerate() {
            let rgb = parse_hex(color).ok_or_else(|| KeyboardError::InvalidSetting(format!("'{color}' isn't a color")))?;
            bytes[light * 3..light * 3 + 3].copy_from_slice(&rgb);
        }
        Ok(bytes)
    }
}

/// Reads a saved pattern, trusting it only when two reads in a row match.
pub fn read_pattern(transport: &mut dyn Transport, slot: u8) -> Result<Pattern, KeyboardError> {
    read_bytes(transport, slot).map(|bytes| Pattern::from_bytes(&bytes))
}

/// The pattern's stored color bytes, from two matching reads.
fn read_bytes(transport: &mut dyn Transport, slot: u8) -> Result<[u8; PATTERN_BYTES], KeyboardError> {
    check_pattern_slot(slot)?;
    for attempt in 1..=READ_ATTEMPTS {
        let first = read_raw(transport, slot)?;
        let second = read_raw(transport, slot)?;
        if first == second {
            let mut bytes = [0u8; PATTERN_BYTES];
            bytes.copy_from_slice(&first[..PATTERN_BYTES]);
            return Ok(bytes);
        }
        log::debug!("pattern {} reads disagreed (attempt {attempt}), reading again", slot + 1);
    }
    log::warn!("pattern {} kept reading differently; another app may be using the keyboard", slot + 1);
    Err(KeyboardError::BadReply { command: "GET_USERPIC", reason: "the pattern kept reading differently".into() })
}

fn read_raw(transport: &mut dyn Transport, slot: u8) -> Result<Vec<u8>, KeyboardError> {
    let mut bytes = Vec::with_capacity(usize::from(PATTERN_READ_PAGES) * 64);
    for page in 0..PATTERN_READ_PAGES {
        bytes.extend(transport::exchange(transport, Command::GetPattern { slot, page })?.reply);
    }
    Ok(bytes)
}

/// Saves a pattern to the keyboard's memory unless it already holds it, then confirms it by reading
/// it back (comparing bytes, so color case doesn't matter), sending it once more at most.
pub fn write_pattern(transport: &mut dyn Transport, slot: u8, pattern: &Pattern) -> Result<Pattern, KeyboardError> {
    check_pattern_slot(slot)?;
    let bytes = pattern.to_bytes()?;
    if read_bytes(transport, slot)? == bytes {
        log::info!("pattern {} already matches; nothing written", slot + 1);
        return Ok(Pattern::from_bytes(&bytes));
    }
    let send = |transport: &mut dyn Transport| {
        for packet in 0..PATTERN_WRITE_PACKETS {
            let start = usize::from(packet) * PATTERN_CHUNK_LEN;
            let end = (start + PATTERN_CHUNK_LEN).min(PATTERN_BYTES);
            let mut chunk = PatternChunk::default();
            chunk.0[..end - start].copy_from_slice(&bytes[start..end]);
            transport::send_write(transport, Command::SetPattern { slot, packet, chunk })?;
        }
        Ok(())
    };
    let kept = |transport: &mut dyn Transport| Ok((read_bytes(transport, slot)? == bytes).then(|| Pattern::from_bytes(&bytes)));
    log::info!("saving pattern {}", slot + 1);
    transport::write_confirmed(transport, "pattern", send, kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::protocol::{PATTERN_SLOTS, REPORT_LEN};

    /// Fake keyboard holding 5 patterns: writes land by packet and reads return raw pages with no echo.
    /// `flip_reads` makes that many reads come back wrong, like a reply from another app.
    struct FakeKeyboard {
        slots: Vec<Vec<u8>>,
        request: Vec<u8>,
        writes: usize,
        drop_writes: usize,
        flip_reads: usize,
    }

    impl FakeKeyboard {
        fn new() -> Self {
            Self { slots: vec![vec![0; 384]; 5], request: Vec::new(), writes: 0, drop_writes: 0, flip_reads: 0 }
        }
    }

    impl Transport for FakeKeyboard {
        fn send_feature(&mut self, report: &[u8]) -> Result<(), KeyboardError> {
            self.request = report[1..].to_vec();
            if self.request[0] == 0x0C {
                self.writes += 1;
                if self.drop_writes > 0 {
                    self.drop_writes -= 1;
                    return Ok(());
                }
                let (slot, packet, len) = (usize::from(self.request[1]), usize::from(self.request[3]), usize::from(self.request[4]));
                let start = packet * PATTERN_CHUNK_LEN;
                let data = self.request[8..8 + len].to_vec();
                self.slots[slot][start..start + len].copy_from_slice(&data);
            }
            Ok(())
        }

        fn get_feature(&mut self, buf: &mut [u8]) -> Result<usize, KeyboardError> {
            assert_eq!(self.request[0], 0x8C, "only pattern reads are expected");
            let (slot, page) = (usize::from(self.request[1]), usize::from(self.request[3]));
            buf[0] = 0;
            buf[1..=REPORT_LEN].copy_from_slice(&self.slots[slot][page * 64..page * 64 + 64]);
            if self.flip_reads > 0 {
                self.flip_reads -= 1;
                buf[1] ^= 0xFF;
            }
            Ok(REPORT_LEN + 1)
        }
    }

    fn painted(light: usize, color: &str) -> Pattern {
        let mut colors = vec!["#000000".to_string(); LIGHT_COUNT];
        colors[light] = color.into();
        Pattern(colors)
    }

    #[test]
    fn reads_colors_in_light_order() {
        let mut keyboard = FakeKeyboard::new();
        // Light 14 is W on the MX 8.2: red, as in the factory's first pattern.
        keyboard.slots[0][42] = 0xFF;
        let pattern = read_pattern(&mut keyboard, 0).unwrap();
        assert_eq!(pattern.0.len(), LIGHT_COUNT);
        assert_eq!(pattern.0[14], "#FF0000");
        assert_eq!(pattern.0[13], "#000000");
    }

    #[test]
    fn a_mixed_up_read_is_read_again() {
        let mut keyboard = FakeKeyboard { flip_reads: 1, ..FakeKeyboard::new() };
        assert_eq!(read_pattern(&mut keyboard, 1).unwrap(), Pattern(vec!["#000000".into(); LIGHT_COUNT]));
    }

    #[test]
    fn saves_and_confirms_a_pattern() {
        let mut keyboard = FakeKeyboard::new();
        let pattern = painted(125, "#12AB34");
        assert_eq!(write_pattern(&mut keyboard, 3, &pattern).unwrap(), pattern);
        assert_eq!(keyboard.writes, 7, "one save is exactly 7 packets");
        assert_eq!(&keyboard.slots[3][375..378], &[0x12, 0xAB, 0x34], "the last light lands in the last packet");
    }

    #[test]
    fn sends_once_more_when_the_keyboard_drops_the_save() {
        let mut keyboard = FakeKeyboard { drop_writes: 7, ..FakeKeyboard::new() };
        write_pattern(&mut keyboard, 0, &painted(0, "#FFFFFF")).unwrap();
        assert_eq!(keyboard.writes, 14);
    }

    #[test]
    fn gives_up_after_the_second_save() {
        let mut keyboard = FakeKeyboard { drop_writes: 14, ..FakeKeyboard::new() };
        assert!(matches!(write_pattern(&mut keyboard, 0, &painted(0, "#FFFFFF")), Err(KeyboardError::NotConfirmed(_))));
        assert_eq!(keyboard.writes, 14);
    }

    #[test]
    fn saving_what_the_keyboard_already_holds_writes_nothing() {
        let mut keyboard = FakeKeyboard::new();
        let pattern = painted(3, "#00FF00");
        write_pattern(&mut keyboard, 1, &pattern).unwrap();
        assert_eq!(keyboard.writes, 7);
        assert_eq!(write_pattern(&mut keyboard, 1, &pattern).unwrap(), pattern);
        assert_eq!(keyboard.writes, 7, "the second save found nothing to change");
    }

    #[test]
    fn a_lowercase_color_saves_once_and_confirms() {
        let mut keyboard = FakeKeyboard::new();
        let saved = write_pattern(&mut keyboard, 0, &painted(9, "#ff00aa")).unwrap();
        assert_eq!(keyboard.writes, 7);
        assert_eq!(saved, painted(9, "#FF00AA"));
    }

    #[test]
    fn a_bad_pattern_writes_nothing() {
        let mut keyboard = FakeKeyboard::new();
        assert!(write_pattern(&mut keyboard, 0, &Pattern(vec!["#000000".into(); 3])).is_err());
        assert!(write_pattern(&mut keyboard, 0, &painted(5, "red")).is_err());
        assert!(write_pattern(&mut keyboard, PATTERN_SLOTS, &painted(5, "#FF0000")).is_err());
        assert_eq!(keyboard.writes, 0);
    }
}
