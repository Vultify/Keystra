//! Message format for the keyboard's RY5088 firmware: 64-byte feature reports with a one-byte checksum.
//! Field layouts are documented in research/PROTOCOL-NOTES.md.

use super::error::KeyboardError;

pub const VENDOR_ID: u16 = 0x046A;
pub const PRODUCT_ID: u16 = 0x012D;
pub const USAGE_PAGE: u16 = 0xFFFF;
pub const USAGE: u16 = 0x0002;

/// Length of a report, not counting the report ID byte hidapi adds in front.
pub const REPORT_LEN: usize = 64;

/// Command bytes that must never reach the keyboard (reset, calibration, bootloaders, flash and
/// firmware upgrades), checked when a request is built and again on the bytes at the send point.
/// Several have the high bit set, so the high bit never proves a command is a harmless read.
pub const BLOCKED_COMMANDS: [u8; 15] = [
    0x01, // factory reset
    0x1C, 0x1E, // magnetism calibration
    0x30, 0x31, 0xB0, 0xB1, // OLED bootloader
    0x32, 0xB2, // screen flash data
    0x50, // SET_SKU
    0x7F, // enter USB bootloader (firmware update)
    0xAC, // flash chip erase
    0xBA, // firmware upgrade data
    0xF7, 0xF8, // wireless receiver status and receiver bootloader
];

/// The keyboard's onboard profiles are numbered 0 to ONBOARD_SLOTS - 1.
pub const ONBOARD_SLOTS: u8 = 4;

/// Where the checksum byte goes. Reads and most writes use Bit7; lighting writes use Bit8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checksum {
    Bit7,
    Bit8,
}

/// The only commands Keystra can send: four reads and the three writes it has decoded.
// Tests list the variants with strum's EnumIter, so a new command can't slip past the safety tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum Command {
    GetDeviceInfo,
    GetProfile,
    GetPollingRate,
    GetLighting,
    /// Switches to an onboard profile slot.
    SetProfile(u8),
    /// Sets the polling rate by firmware code (see `settings::POLLING_RATES`).
    SetPollingRate(u8),
    /// Bytes 1 to 7 of a lighting write, built by `LightingSettings::encode`.
    SetLighting([u8; 7]),
}

impl Command {
    pub const fn id(self) -> u8 {
        match self {
            Command::GetDeviceInfo => 0x8F,
            Command::GetProfile => 0x84,
            Command::GetPollingRate => 0x83,
            Command::GetLighting => 0x87,
            Command::SetProfile(_) => 0x04,
            Command::SetPollingRate(_) => 0x03,
            Command::SetLighting(_) => 0x07,
        }
    }

    pub const fn checksum(self) -> Checksum {
        match self {
            Command::SetLighting(_) => Checksum::Bit8,
            _ => Checksum::Bit7,
        }
    }

    pub const fn is_write(self) -> bool {
        matches!(self, Command::SetProfile(_) | Command::SetPollingRate(_) | Command::SetLighting(_))
    }

    pub const fn name(self) -> &'static str {
        match self {
            Command::GetDeviceInfo => "GET_USB_VERSION",
            Command::GetProfile => "GET_PROFILE",
            Command::GetPollingRate => "GET_REPORT",
            Command::GetLighting => "GET_LEDPARAM",
            Command::SetProfile(_) => "SET_PROFILE",
            Command::SetPollingRate(_) => "SET_REPORT",
            Command::SetLighting(_) => "SET_LEDPARAM",
        }
    }
}

/// Writes the checksum into its slot: 255 minus the low byte of the sum of the bytes before it.
pub fn apply_checksum(report: &mut [u8; REPORT_LEN], kind: Checksum) {
    let slot = match kind {
        Checksum::Bit7 => 7,
        Checksum::Bit8 => 8,
    };
    let sum = report[..slot].iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
    report[slot] = 255 - sum;
}

/// Builds the 64-byte request for a command, refusing anything on the never-send list.
pub fn build_request(command: Command) -> Result<[u8; REPORT_LEN], KeyboardError> {
    let id = command.id();
    if BLOCKED_COMMANDS.contains(&id) {
        return Err(KeyboardError::Blocked(id));
    }
    let mut report = [0u8; REPORT_LEN];
    report[0] = id;
    match command {
        Command::SetProfile(slot) if slot >= ONBOARD_SLOTS => {
            return Err(KeyboardError::InvalidSetting(format!("there is no onboard profile {}", slot + 1)));
        }
        Command::SetProfile(slot) => report[1] = slot,
        // The rate code goes in byte 2 for both SET and GET.
        Command::SetPollingRate(code) => report[2] = code,
        Command::SetLighting(bytes) => report[1..8].copy_from_slice(&bytes),
        _ => {}
    }
    apply_checksum(&mut report, command.checksum());
    Ok(report)
}

/// Refuses outgoing bytes whose command is on the never-send list, at the send point so it covers
/// any future builder too. `report` starts with the report ID, so the command is byte 1.
pub fn guard_outgoing(report: &[u8]) -> Result<(), KeyboardError> {
    match report.get(1) {
        Some(id) if BLOCKED_COMMANDS.contains(id) => Err(KeyboardError::Blocked(*id)),
        Some(_) => Ok(()),
        None => Err(KeyboardError::Hid("outgoing report has no command byte".into())),
    }
}

/// Confirms a reply echoes the command it answers.
pub fn check_echo(command: Command, reply: &[u8]) -> Result<(), KeyboardError> {
    match reply.first() {
        Some(&byte) if byte == command.id() => Ok(()),
        Some(&byte) => Err(KeyboardError::BadReply {
            command: command.name(),
            reason: format!("expected echo 0x{:02X}, got 0x{byte:02X}", command.id()),
        }),
        None => Err(KeyboardError::BadReply {
            command: command.name(),
            reason: "empty reply".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn bit7_checksum_matches_known_packets() {
        // GET_USB_VERSION: 255 - 0x8F = 0x70.
        let request = build_request(Command::GetDeviceInfo).unwrap();
        assert_eq!(&request[..8], &[0x8F, 0, 0, 0, 0, 0, 0, 0x70]);
        assert!(request[8..].iter().all(|b| *b == 0));

        // Documented RY5088 example: F7 -> checksum 08.
        let mut report = [0u8; REPORT_LEN];
        report[0] = 0xF7;
        apply_checksum(&mut report, Checksum::Bit7);
        assert_eq!(report[7], 0x08);
    }

    #[test]
    fn checksum_wraps_when_the_sum_overflows() {
        let mut report = [0u8; REPORT_LEN];
        report[..7].copy_from_slice(&[0xFF, 0xFF, 0x02, 0, 0, 0, 0]);
        apply_checksum(&mut report, Checksum::Bit7);
        // 0xFF + 0xFF + 0x02 = 0x200, low byte 0x00, so the checksum is 0xFF.
        assert_eq!(report[7], 0xFF);
    }

    #[test]
    fn bit8_checksum_covers_eight_bytes() {
        let mut report = [0u8; REPORT_LEN];
        report[..8].copy_from_slice(&[0x07, 0x05, 0x01, 0x04, 0x07, 0xFF, 0x00, 0x00]);
        apply_checksum(&mut report, Checksum::Bit8);
        let sum: u32 = report[..8].iter().map(|b| u32::from(*b)).sum();
        assert_eq!(report[8], 255 - (sum & 0xFF) as u8);
        assert_eq!(report[7], 0x00, "Bit8 must not touch byte 7");
    }

    #[test]
    fn every_read_request_matches_its_command() {
        for command in Command::iter().filter(|c| !c.is_write()) {
            let request = build_request(command).unwrap();
            assert_eq!(request[0], command.id());
            assert_eq!(request[7], 255 - command.id());
        }
    }

    #[test]
    fn writes_put_their_payload_where_magcrate_does() {
        let switch = build_request(Command::SetProfile(2)).unwrap();
        assert_eq!(&switch[..8], &[0x04, 2, 0, 0, 0, 0, 0, 255 - 0x06]);
        let polling = build_request(Command::SetPollingRate(2)).unwrap();
        assert_eq!(&polling[..8], &[0x03, 0, 2, 0, 0, 0, 0, 255 - 0x05]);
        let lighting = build_request(Command::SetLighting([0x10, 0x02, 0x04, 0x08, 0xFA, 0xFF, 0xFA])).unwrap();
        assert_eq!(&lighting[..8], &[0x07, 0x10, 0x02, 0x04, 0x08, 0xFA, 0xFF, 0xFA]);
        let sum = lighting[..8].iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
        assert_eq!(lighting[8], 255 - sum, "lighting writes use the Bit8 checksum");
        assert!(lighting[9..].iter().all(|b| *b == 0));
    }

    #[test]
    fn switching_to_a_slot_that_does_not_exist_is_refused() {
        assert!(build_request(Command::SetProfile(ONBOARD_SLOTS)).is_err());
    }

    #[test]
    fn no_allowed_command_is_on_the_never_send_list() {
        for command in Command::iter() {
            assert!(
                !BLOCKED_COMMANDS.contains(&command.id()),
                "{} (0x{:02X}) is blocked",
                command.name(),
                command.id()
            );
        }
    }

    #[test]
    fn allowlist_is_exactly_the_decoded_commands() {
        // Listed by value on purpose: the high bit alone doesn't prove a command is a read
        // (0xAC flash erase and 0xBA firmware data both have it).
        let ids: Vec<u8> = Command::iter().map(|c| c.id()).collect();
        assert_eq!(ids, [0x8F, 0x84, 0x83, 0x87, 0x04, 0x03, 0x07]);
        let writes: Vec<u8> = Command::iter().filter(|c| c.is_write()).map(|c| c.id()).collect();
        assert_eq!(writes, [0x04, 0x03, 0x07]);
    }

    #[test]
    fn send_point_guard_refuses_blocked_bytes() {
        for id in BLOCKED_COMMANDS {
            let mut report = [0u8; REPORT_LEN + 1];
            report[1] = id;
            assert!(
                matches!(guard_outgoing(&report), Err(KeyboardError::Blocked(b)) if b == id),
                "0x{id:02X} got through"
            );
        }
        assert!(guard_outgoing(&[0x00, 0x84]).is_ok());
        assert!(guard_outgoing(&[0x00]).is_err());
    }

    #[test]
    fn firmware_update_entry_is_blocked() {
        for id in [0x7F, 0xBA, 0xF8, 0xAC, 0x01] {
            assert!(BLOCKED_COMMANDS.contains(&id), "0x{id:02X} must be blocked");
        }
    }

    #[test]
    fn echo_check_rejects_wrong_or_empty_replies() {
        assert!(check_echo(Command::GetProfile, &[0x84, 0x01]).is_ok());
        assert!(check_echo(Command::GetProfile, &[0x83, 0x01]).is_err());
        assert!(check_echo(Command::GetProfile, &[]).is_err());
    }
}
