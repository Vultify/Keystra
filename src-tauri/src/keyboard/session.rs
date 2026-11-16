//! What Keystra reads from and writes to an open keyboard: the full settings read, and applying
//! changes with a read-back. Works on any `Transport`, so it's tested without USB.

use serde::Serialize;

use super::error::KeyboardError;
use super::lighting::{self, Lighting, LightingSettings};
use super::protocol::{Command, ONBOARD_SLOTS};
use super::settings::{self, DeviceInfo};
use super::transport::{self, Exchange, Transport};

/// Everything Keystra reads from the keyboard, plus the raw bytes behind it.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub info: DeviceInfo,
    /// The active onboard profile, counting from 0.
    pub onboard_slot: u8,
    pub polling_rate: Option<u32>,
    pub lighting: Lighting,
    pub exchanges: Vec<Exchange>,
}

/// What to change on the keyboard. Parts left as `None` stay as they are.
#[derive(Clone, Debug, Default)]
pub struct ApplyRequest {
    pub onboard_slot: Option<u8>,
    pub polling_rate_hz: Option<u32>,
    pub lighting: Option<LightingSettings>,
}

impl ApplyRequest {
    /// Combines this request with a newer one: the newer one wins wherever it sets a part.
    pub fn then(self, newer: ApplyRequest) -> ApplyRequest {
        ApplyRequest {
            onboard_slot: newer.onboard_slot.or(self.onboard_slot),
            polling_rate_hz: newer.polling_rate_hz.or(self.polling_rate_hz),
            lighting: newer.lighting.or(self.lighting),
        }
    }
}

/// Runs the reads in a fixed order and parses each reply.
pub fn read_settings(transport: &mut dyn Transport, info: DeviceInfo) -> Result<Snapshot, KeyboardError> {
    let device = transport::exchange(transport, Command::GetDeviceInfo)?;
    let slot = transport::exchange(transport, Command::GetProfile)?;
    let polling = transport::exchange(transport, Command::GetPollingRate)?;
    let lighting = transport::exchange(transport, Command::GetLighting)?;

    let fresh_info = settings::parse_device_info(&device.reply).unwrap_or(info);
    Ok(Snapshot {
        info: fresh_info,
        onboard_slot: settings::parse_onboard_slot(&slot.reply)?,
        polling_rate: settings::parse_polling_rate(&polling.reply)?,
        lighting: lighting::parse_lighting(&lighting.reply)?,
        exchanges: vec![device, slot, polling, lighting],
    })
}

/// Writes the parts of `request` that differ from the keyboard, in the order slot, polling rate,
/// lighting. Every value is checked before anything is sent, so a bad one changes nothing.
pub fn apply_settings(
    transport: &mut dyn Transport,
    info: DeviceInfo,
    request: &ApplyRequest,
) -> Result<Snapshot, KeyboardError> {
    if let Some(slot) = request.onboard_slot.filter(|&slot| slot >= ONBOARD_SLOTS) {
        return Err(KeyboardError::InvalidSetting(format!("there is no onboard profile {}", slot + 1)));
    }
    let polling_code = request.polling_rate_hz.map(settings::polling_code).transpose()?;
    let lighting_bytes = request.lighting.as_ref().map(LightingSettings::encode).transpose()?;

    let before = read_settings(transport, info.clone())?;
    if let Some(slot) = request.onboard_slot.filter(|&slot| slot != before.onboard_slot) {
        write_setting(transport, Command::SetProfile(slot), "onboard profile", |t| {
            Ok(settings::parse_onboard_slot(&transport::exchange(t, Command::GetProfile)?.reply)? == slot)
        })?;
    }
    if let (Some(hz), Some(code)) = (request.polling_rate_hz, polling_code)
        && before.polling_rate != Some(hz)
    {
        write_setting(transport, Command::SetPollingRate(code), "polling rate", |t| {
            Ok(settings::parse_polling_rate(&transport::exchange(t, Command::GetPollingRate)?.reply)? == Some(hz))
        })?;
    }
    if let (Some(wanted), Some(bytes)) = (&request.lighting, lighting_bytes)
        && !before.lighting.matches(wanted)
    {
        write_setting(transport, Command::SetLighting(bytes), "lighting", |t| {
            Ok(lighting::parse_lighting(&transport::exchange(t, Command::GetLighting)?.reply)?.matches(wanted))
        })?;
    }
    read_settings(transport, info)
}

/// Sends one setting write and confirms it with `kept`, resending once at most.
fn write_setting(
    transport: &mut dyn Transport,
    command: Command,
    what: &'static str,
    mut kept: impl FnMut(&mut dyn Transport) -> Result<bool, KeyboardError>,
) -> Result<(), KeyboardError> {
    transport::write_confirmed(transport, what, |t| transport::send_write(t, command), |t| Ok(kept(t)?.then_some(())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::protocol::REPORT_LEN;

    /// GET_USB_VERSION reply from an MX 8.2: device 2494, firmware 0x0104.
    const DEVICE_INFO_REPLY: [u8; 9] = [0x8F, 0xBE, 0x09, 0, 0, 0, 0, 0x04, 0x01];

    /// Fake MX 8.2 that keeps what writes set and reports it back. `ignore_writes` drops that many
    /// writes first, like a keyboard busy with another app.
    struct FakeKeyboard {
        last: u8,
        slot: u8,
        polling_code: u8,
        lighting: [u8; 7],
        ignore_writes: usize,
        writes: Vec<u8>,
    }

    impl FakeKeyboard {
        fn new() -> Self {
            Self {
                last: 0,
                slot: 0,
                polling_code: 2,
                lighting: [0x10, 0x02, 0x04, 0x08, 0xFA, 0xFF, 0xFA],
                ignore_writes: 0,
                writes: Vec::new(),
            }
        }
    }

    impl Transport for FakeKeyboard {
        fn send_feature(&mut self, report: &[u8]) -> Result<(), KeyboardError> {
            let command = report[1];
            self.last = command;
            if matches!(command, 0x03 | 0x04 | 0x07) {
                self.writes.push(command);
                if self.ignore_writes > 0 {
                    self.ignore_writes -= 1;
                    return Ok(());
                }
                match command {
                    0x04 => self.slot = report[2],
                    0x03 => self.polling_code = report[3],
                    _ => self.lighting.copy_from_slice(&report[2..9]),
                }
            }
            Ok(())
        }

        fn get_feature(&mut self, buf: &mut [u8]) -> Result<usize, KeyboardError> {
            let body: Vec<u8> = match self.last {
                0x8F => DEVICE_INFO_REPLY.to_vec(),
                0x84 => vec![0x84, self.slot],
                0x83 => vec![0x83, 0x00, self.polling_code],
                0x87 => [&[0x87][..], &self.lighting].concat(),
                other => panic!("unexpected read after 0x{other:02X}"),
            };
            buf[0] = 0;
            buf[1..=body.len()].copy_from_slice(&body);
            Ok(REPORT_LEN + 1)
        }
    }

    fn info() -> DeviceInfo {
        settings::parse_device_info(&DEVICE_INFO_REPLY).unwrap()
    }

    fn red_ripple() -> LightingSettings {
        LightingSettings { mode: 5, brightness: 3, speed: 1, direction: 0, rainbow: false, color: "#FF0000".into() }
    }

    #[test]
    fn reads_every_setting_in_order() {
        let mut keyboard = FakeKeyboard { lighting: [5, 1, 4, 0x07, 0xFA, 0xFF, 0xFA], ..FakeKeyboard::new() };
        let snapshot = read_settings(&mut keyboard, info()).unwrap();
        assert_eq!(snapshot.info.device_id, 2494);
        assert_eq!(snapshot.onboard_slot, 0);
        assert_eq!(snapshot.polling_rate, Some(2000));
        assert_eq!(snapshot.lighting.mode_id, 5);
        assert_eq!(snapshot.lighting.color.as_deref(), Some("#FFFFFF"));
        let order: Vec<&str> = snapshot.exchanges.iter().map(|e| e.name).collect();
        assert_eq!(order, ["GET_USB_VERSION", "GET_PROFILE", "GET_REPORT", "GET_LEDPARAM"]);
        assert!(keyboard.writes.is_empty());
    }

    #[test]
    fn applies_and_confirms_every_part() {
        let mut keyboard = FakeKeyboard::new();
        let request = ApplyRequest { onboard_slot: Some(2), polling_rate_hz: Some(8000), lighting: Some(red_ripple()) };
        let snapshot = apply_settings(&mut keyboard, info(), &request).unwrap();
        assert_eq!(snapshot.onboard_slot, 2);
        assert_eq!(snapshot.polling_rate, Some(8000));
        assert!(snapshot.lighting.matches(&red_ripple()));
        assert_eq!(keyboard.writes, [0x04, 0x03, 0x07], "slot, then polling, then lighting");
    }

    #[test]
    fn writes_only_what_differs() {
        let mut keyboard = FakeKeyboard::new();
        let request = ApplyRequest {
            onboard_slot: Some(0),
            polling_rate_hz: Some(2000),
            lighting: Some(LightingSettings::factory()),
        };
        apply_settings(&mut keyboard, info(), &request).unwrap();
        assert!(keyboard.writes.is_empty(), "the keyboard already had all of it");
    }

    #[test]
    fn resends_once_when_the_keyboard_drops_a_write() {
        let mut keyboard = FakeKeyboard { ignore_writes: 1, ..FakeKeyboard::new() };
        let request = ApplyRequest { polling_rate_hz: Some(1000), ..ApplyRequest::default() };
        apply_settings(&mut keyboard, info(), &request).unwrap();
        assert_eq!(keyboard.writes, [0x03, 0x03]);
    }

    #[test]
    fn gives_up_after_the_second_try() {
        let mut keyboard = FakeKeyboard { ignore_writes: 5, ..FakeKeyboard::new() };
        let request = ApplyRequest { onboard_slot: Some(3), ..ApplyRequest::default() };
        let result = apply_settings(&mut keyboard, info(), &request);
        assert!(matches!(result, Err(KeyboardError::NotConfirmed("onboard profile"))));
        assert_eq!(keyboard.writes.len(), 2);
    }

    #[test]
    fn a_bad_value_stops_everything_before_any_write() {
        let mut keyboard = FakeKeyboard::new();
        let mut music3 = red_ripple();
        music3.mode = 20;
        for request in [
            ApplyRequest { onboard_slot: Some(1), polling_rate_hz: Some(3000), ..ApplyRequest::default() },
            ApplyRequest { onboard_slot: Some(1), lighting: Some(music3), ..ApplyRequest::default() },
            ApplyRequest { onboard_slot: Some(ONBOARD_SLOTS), ..ApplyRequest::default() },
        ] {
            assert!(apply_settings(&mut keyboard, info(), &request).is_err());
        }
        assert!(keyboard.writes.is_empty());
    }

    #[test]
    fn a_newer_request_wins_only_where_it_sets_a_part() {
        let older = ApplyRequest { onboard_slot: Some(1), polling_rate_hz: Some(1000), lighting: None };
        let newer = ApplyRequest { polling_rate_hz: Some(8000), lighting: Some(red_ripple()), ..ApplyRequest::default() };
        let merged = older.then(newer);
        assert_eq!(merged.onboard_slot, Some(1));
        assert_eq!(merged.polling_rate_hz, Some(8000));
        assert_eq!(merged.lighting, Some(red_ripple()));
    }
}
