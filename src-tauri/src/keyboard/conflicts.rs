//! Spots other apps that use the keyboard's settings channel, like MagCrate. The keyboard has one
//! reply slot, so two apps asking at once can pick up each other's answers.

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

/// True while MagCrate or its USB helper is running. Reads names from a one-off process snapshot,
/// so no process is ever opened and nothing is kept between checks.
pub fn magcrate_running() -> bool {
    let Some(snapshot) = Snapshot::take() else {
        return false;
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: the snapshot handle is valid until dropped, and `entry` is a correctly sized struct.
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
    while more {
        if is_magcrate(&exe_name(&entry.szExeFile)) {
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

fn is_magcrate(process_name: &str) -> bool {
    let name = process_name.to_ascii_lowercase();
    name.contains("magcrate") || name.starts_with("iot_driver")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_magcrate_and_its_helper() {
        assert!(is_magcrate("MagCrate Driver.exe"));
        assert!(is_magcrate("iot_driver_v207.exe"));
        assert!(!is_magcrate("keystra.exe"));
        assert!(!is_magcrate("explorer.exe"));
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
        let _ = magcrate_running();
    }
}
