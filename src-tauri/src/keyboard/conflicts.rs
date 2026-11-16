//! Spots other apps that use the keyboard's settings channel, like MagCrate. The keyboard has one
//! reply slot, so two apps asking at once can pick up each other's answers.

use serde::Serialize;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

/// What of MagCrate is running. Its helper (iot_driver) can outlive the app and usually sits idle,
/// so it gets a softer warning than MagCrate itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub enum MagCrateState {
    Closed,
    HelperOnly,
    Open,
}

/// Looks for MagCrate in a one-off process snapshot, so no process is ever opened and nothing is
/// kept between checks.
pub fn magcrate_state() -> MagCrateState {
    let Some(snapshot) = Snapshot::take() else {
        return MagCrateState::Closed;
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut state = MagCrateState::Closed;
    // SAFETY: the snapshot handle is valid until dropped, and `entry` is a correctly sized struct.
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
    while more {
        match classify(&exe_name(&entry.szExeFile)) {
            MagCrateState::Open => return MagCrateState::Open,
            MagCrateState::HelperOnly => state = MagCrateState::HelperOnly,
            MagCrateState::Closed => {}
        }
        // SAFETY: same as above.
        more = unsafe { Process32NextW(snapshot.0, &mut entry) } != 0;
    }
    state
}

/// Whether another Keystra process is running, like the installed app while a development build
/// is open. Two copies driving one keyboard would undo each other's settings.
pub fn other_keystra_running() -> bool {
    let Some(snapshot) = Snapshot::take() else {
        return false;
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let own = std::process::id();
    // SAFETY: the snapshot handle is valid until dropped, and `entry` is a correctly sized struct.
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
    while more {
        if entry.th32ProcessID != own && exe_name(&entry.szExeFile).eq_ignore_ascii_case("keystra.exe") {
            return true;
        }
        // SAFETY: same as above.
        more = unsafe { Process32NextW(snapshot.0, &mut entry) } != 0;
    }
    false
}

/// A Windows process snapshot handle, closed when dropped.
struct Snapshot(HANDLE);

impl Snapshot {
    fn take() -> Option<Self> {
        // SAFETY: plain Win32 call with constant arguments; failure is checked below.
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        (handle != INVALID_HANDLE_VALUE).then_some(Self(handle))
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        // SAFETY: the handle came from CreateToolhelp32Snapshot and is closed exactly once.
        unsafe { CloseHandle(self.0) };
    }
}

fn exe_name(raw: &[u16]) -> String {
    let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
    String::from_utf16_lossy(&raw[..len])
}

/// Sorts one process name: MagCrate's app, its USB helper, or something else.
fn classify(process_name: &str) -> MagCrateState {
    let name = process_name.to_ascii_lowercase();
    if name.contains("magcrate") {
        MagCrateState::Open
    } else if name.starts_with("iot_driver") {
        MagCrateState::HelperOnly
    } else {
        MagCrateState::Closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_magcrate_apart_from_its_helper() {
        assert_eq!(classify("MagCrate Driver.exe"), MagCrateState::Open);
        assert_eq!(classify("iot_driver_v207.exe"), MagCrateState::HelperOnly);
        assert_eq!(classify("keystra.exe"), MagCrateState::Closed);
        assert_eq!(classify("explorer.exe"), MagCrateState::Closed);
    }

    #[test]
    fn exe_name_stops_at_the_terminator() {
        let mut raw = [0u16; 16];
        for (slot, c) in raw.iter_mut().zip("abc.exe".encode_utf16()) {
            *slot = c;
        }
        assert_eq!(exe_name(&raw), "abc.exe");
    }

    #[test]
    fn snapshot_scan_runs() {
        // Smoke test against the real system: must not crash, and this test runner isn't MagCrate.
        let _ = magcrate_state();
    }
}
