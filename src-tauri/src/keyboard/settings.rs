//! Turns raw replies into device info, the active onboard profile and the polling rate. Lighting
//! has its own module.

use serde::Serialize;

use super::error::KeyboardError;
use super::protocol::Command;

/// Device IDs reported by GET_USB_VERSION for the MX 8.2 Pro TMR.
const MODELS: [(u32, &str); 2] = [(2494, "MX 8.2 Pro TMR"), (3098, "MX 8.2 Pro TMR (DE)")];

/// Polling rates in Hz by firmware code: the code is the index. Shared by reads and writes.
pub const POLLING_RATES: [u32; 7] = [8000, 4000, 2000, 1000, 500, 250, 125];

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

pub(super) fn field(command: Command, reply: &[u8], index: usize) -> Result<u8, KeyboardError> {
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

/// Returns the polling rate in Hz, or `None` for a code the firmware doesn't document.
pub fn parse_polling_rate(reply: &[u8]) -> Result<Option<u32>, KeyboardError> {
    let code = field(Command::GetPollingRate, reply, 2)?;
    Ok(POLLING_RATES.get(usize::from(code)).copied())
}

/// The firmware code for a polling rate, the other half of `parse_polling_rate`.
pub fn polling_code(hz: u32) -> Result<u8, KeyboardError> {
    POLLING_RATES
        .iter()
        .position(|&rate| rate == hz)
        .and_then(|code| u8::try_from(code).ok())
        .ok_or_else(|| KeyboardError::InvalidSetting(format!("{hz} Hz isn't a polling rate the keyboard supports")))
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
        assert!(parse_polling_rate(&[0x83]).is_err());
    }

    #[test]
    fn polling_codes_map_to_hz_and_back() {
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 2])).unwrap(), Some(2000));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 0])).unwrap(), Some(8000));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 6])).unwrap(), Some(125));
        assert_eq!(parse_polling_rate(&reply(&[0x83, 0, 9])).unwrap(), None);
        for (code, hz) in POLLING_RATES.iter().enumerate() {
            assert_eq!(usize::from(polling_code(*hz).unwrap()), code);
        }
        assert!(polling_code(3000).is_err());
    }

    #[test]
    fn profile_is_byte_one() {
        assert_eq!(parse_onboard_slot(&reply(&[0x84, 2])).unwrap(), 2);
    }
}
