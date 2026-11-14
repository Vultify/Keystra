//! The one owner of the keyboard. A single thread holds the USB handle, runs every request in
//! order, notices plugs and unplugs, and tells the screens when the connection changes.

use std::ffi::{CStr, CString};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::HidApi;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::conflicts::{self, MagCrateState};
use super::error::KeyboardError;
use super::protocol::{Command, PRODUCT_ID, USAGE, USAGE_PAGE, VENDOR_ID};
use super::session::{self, ApplyRequest, Snapshot};
use super::settings::{self, DeviceInfo};
use super::transport::{self, HidTransport, Transport};

/// How often the worker checks whether the keyboard was plugged in or pulled out.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// While connected, MagCrate is looked for every this many polls.
const CONFLICT_CHECK_EVERY: u32 = 3;

/// Waits before retrying a keyboard that failed to answer, growing with each failure.
const RETRY_DELAYS: [Duration; 4] = [
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(30),
];

/// The least time between two applies, so quick repeated changes can never become a stream of
/// writes. Requests that arrive meanwhile are merged into one.
const MIN_WRITE_GAP: Duration = Duration::from_millis(500);

/// Shown when the keyboard is there but its replies keep getting mixed up.
const BUSY_MESSAGE: &str = "the keyboard is busy, another app may be using it";

/// Event the screens listen to for connection changes.
pub const STATUS_EVENT: &str = "keyboard-status";

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(tag = "state", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum KeyboardStatus {
    Disconnected,
    /// `magcrate` warns when MagCrate, which shares the keyboard and can mix up replies, is running.
    Connected { info: DeviceInfo, magcrate: MagCrateState },
    Unsupported { device_id: u32 },
    Error { message: String },
}

enum Job {
    ReadAll(Sender<Result<Snapshot, KeyboardError>>),
    Apply(ApplyRequest, Sender<Result<Snapshot, KeyboardError>>),
}

/// Cheap handle the commands use to talk to the worker thread.
#[derive(Clone)]
pub struct KeyboardHandle {
    jobs: Sender<Job>,
    status: Arc<Mutex<KeyboardStatus>>,
}

impl KeyboardHandle {
    pub fn spawn(app: AppHandle) -> Self {
        let (jobs, inbox) = mpsc::channel();
        let status = Arc::new(Mutex::new(KeyboardStatus::Disconnected));
        let worker = Worker {
            app,
            status: Arc::clone(&status),
            api: None,
            device: None,
            skips: Skips::default(),
            magcrate: MagCrateState::Closed,
            polls: 0,
            last_apply: None,
        };
        thread::Builder::new()
            .name("keyboard-worker".into())
            .spawn(move || worker.run(inbox))
            .expect("failed to start the keyboard worker thread");
        Self { jobs, status }
    }

    pub fn status(&self) -> KeyboardStatus {
        self.status.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status(), KeyboardStatus::Connected { .. })
    }

    /// Reads every setting Keystra knows in one go. Blocks until the worker answers.
    pub fn read_all(&self) -> Result<Snapshot, KeyboardError> {
        let (reply, answer) = mpsc::channel();
        self.jobs.send(Job::ReadAll(reply)).map_err(|_| KeyboardError::WorkerStopped)?;
        answer.recv().map_err(|_| KeyboardError::WorkerStopped)?
    }

    /// Writes whatever in `request` differs from the keyboard, confirming each write with a read.
    /// Blocks until the worker answers with the keyboard's settings afterwards.
    pub fn apply(&self, request: ApplyRequest) -> Result<Snapshot, KeyboardError> {
        let (reply, answer) = mpsc::channel();
        self.jobs.send(Job::Apply(request, reply)).map_err(|_| KeyboardError::WorkerStopped)?;
        answer.recv().map_err(|_| KeyboardError::WorkerStopped)?
    }
}

struct Connected {
    transport: HidTransport,
    path: CString,
    info: DeviceInfo,
}

/// A keyboard that failed to answer, and when to try it again.
struct Cooldown {
    path: CString,
    retry_at: Instant,
    failures: usize,
}

/// What the worker does after looking for the keyboard.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    Stay,
    Connect,
    Reconnect,
    Disconnect,
    ReportMissing,
}

/// Decides the next step from the open device, the device found now, and a device being skipped
/// (unknown ID or cooling down after a failure). Kept pure so the rules can be tested without USB.
fn next_step(open: Option<&CStr>, found: Option<&CStr>, skipped: Option<&CStr>) -> Step {
    match (open, found) {
        (Some(open), Some(found)) if open == found => Step::Stay,
        (Some(_), Some(_)) => Step::Reconnect,
        (Some(_), None) => Step::Disconnect,
        (None, Some(found)) if skipped == Some(found) => Step::Stay,
        (None, Some(_)) => Step::Connect,
        (None, None) => Step::ReportMissing,
    }
}

fn retry_delay(failures: usize) -> Duration {
    RETRY_DELAYS[failures.saturating_sub(1).min(RETRY_DELAYS.len() - 1)]
}

/// Devices the worker leaves alone for now: one that answered with an unknown ID (until it's
/// unplugged) and one cooling down after failing to answer. No USB here, so it's fully testable.
#[derive(Default)]
struct Skips {
    rejected: Option<CString>,
    cooldown: Option<Cooldown>,
}

impl Skips {
    /// Forgets anything that isn't the device found now, then returns the path to skip this round.
    fn update(&mut self, found: Option<&CStr>, now: Instant) -> Option<&CStr> {
        if self.rejected.as_deref() != found {
            self.rejected = None;
        }
        if self.cooldown.as_ref().is_some_and(|c| Some(c.path.as_c_str()) != found) {
            self.cooldown = None;
        }
        let cooling = self
            .cooldown
            .as_ref()
            .filter(|c| now < c.retry_at)
            .map(|c| c.path.as_c_str());
        self.rejected.as_deref().or(cooling)
    }

    fn reject(&mut self, path: CString) {
        self.rejected = Some(path);
    }

    /// Records a failed connect and returns how long to wait before trying that device again.
    fn record_failure(&mut self, path: CString, now: Instant) -> Duration {
        let earlier = self.cooldown.as_ref().filter(|c| c.path == path).map_or(0, |c| c.failures);
        let failures = earlier + 1;
        let delay = retry_delay(failures);
        self.cooldown = Some(Cooldown {
            path,
            retry_at: now + delay,
            failures,
        });
        delay
    }

    fn clear_failures(&mut self) {
        self.cooldown = None;
    }
}

struct Worker {
    app: AppHandle,
    status: Arc<Mutex<KeyboardStatus>>,
    api: Option<HidApi>,
    device: Option<Connected>,
    skips: Skips,
    magcrate: MagCrateState,
    polls: u32,
    last_apply: Option<Instant>,
}

impl Worker {
    fn run(mut self, inbox: Receiver<Job>) {
        log::info!("keyboard worker started");
        self.refresh();
        loop {
            match inbox.recv_timeout(POLL_INTERVAL) {
                // The caller may have gone away; nothing to do if so.
                Ok(Job::ReadAll(reply)) => {
                    let _ = reply.send(self.with_device(session::read_settings));
                }
                Ok(Job::Apply(request, reply)) => self.apply(request, reply, &inbox),
                Err(RecvTimeoutError::Timeout) => self.refresh(),
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        log::info!("keyboard worker stopped");
    }

    /// Notices plugs and unplugs, connects to the keyboard when it appears, and watches for MagCrate.
    fn refresh(&mut self) {
        let found = match self.find_keyboard() {
            Ok(path) => path,
            Err(err) => {
                self.set_status(KeyboardStatus::Error { message: err.to_string() });
                return;
            }
        };

        let skipped = self.skips.update(found.as_deref(), Instant::now());
        let open = self.device.as_ref().map(|d| d.path.as_c_str());
        match next_step(open, found.as_deref(), skipped) {
            Step::Stay => {
                if self.device.is_some() && self.polls.is_multiple_of(CONFLICT_CHECK_EVERY) {
                    self.check_conflicts();
                }
            }
            Step::Connect => {
                self.check_conflicts();
                self.connect(found.expect("Connect means a device was found"));
            }
            Step::Reconnect => {
                self.disconnect();
                self.check_conflicts();
                self.connect(found.expect("Reconnect means a device was found"));
            }
            Step::Disconnect => self.disconnect(),
            Step::ReportMissing => self.set_status(KeyboardStatus::Disconnected),
        }
        self.polls = self.polls.wrapping_add(1);
    }

    fn check_conflicts(&mut self) {
        let state = conflicts::magcrate_state();
        if state == self.magcrate {
            return;
        }
        self.magcrate = state;
        log::info!("MagCrate is now {state:?}");
        if let Some(device) = &self.device {
            self.set_status(KeyboardStatus::Connected {
                info: device.info.clone(),
                magcrate: state,
            });
        }
    }

    /// Returns the path of the keyboard's settings interface, if it's plugged in.
    fn find_keyboard(&mut self) -> Result<Option<CString>, KeyboardError> {
        if self.api.is_none() {
            self.api = Some(HidApi::new()?);
        }
        let api = self.api.as_mut().ok_or(KeyboardError::NotConnected)?;
        api.reset_devices()?;
        api.add_devices(VENDOR_ID, PRODUCT_ID)?;
        let path = api
            .device_list()
            .find(|d| d.usage_page() == USAGE_PAGE && d.usage() == USAGE)
            .map(|d| d.path().to_owned());
        Ok(path)
    }

    fn connect(&mut self, path: CString) {
        match self.open_and_identify(&path) {
            Ok((transport, info)) if info.is_supported() => {
                log::info!(
                    "connected to {} (device {}, firmware 0x{:04X})",
                    info.model.unwrap_or("keyboard"),
                    info.device_id,
                    info.firmware_version
                );
                self.set_status(KeyboardStatus::Connected {
                    info: info.clone(),
                    magcrate: self.magcrate,
                });
                self.skips.clear_failures();
                self.device = Some(Connected { transport, path, info });
            }
            Ok((_, info)) => {
                log::warn!("ignoring device with unknown ID {}", info.device_id);
                self.skips.reject(path);
                self.set_status(KeyboardStatus::Unsupported { device_id: info.device_id });
            }
            Err(err) => {
                let delay = self.skips.record_failure(path, Instant::now());
                log::debug!("connect failed ({err}), retrying in {}s", delay.as_secs());
                // A steady message for mixed-up replies, so repeated failures don't flood the log.
                let message = match err {
                    KeyboardError::BadReply { .. } => BUSY_MESSAGE.to_string(),
                    other => other.to_string(),
                };
                self.set_status(KeyboardStatus::Error { message });
            }
        }
    }

    fn open_and_identify(&self, path: &CString) -> Result<(HidTransport, DeviceInfo), KeyboardError> {
        let api = self.api.as_ref().ok_or(KeyboardError::NotConnected)?;
        let mut transport = HidTransport::new(api.open_path(path)?);
        let exchange = transport::exchange(&mut transport, Command::GetDeviceInfo)?;
        let info = settings::parse_device_info(&exchange.reply)?;
        Ok((transport, info))
    }

    fn disconnect(&mut self) {
        if self.device.take().is_some() {
            log::info!("keyboard disconnected");
        }
        self.set_status(KeyboardStatus::Disconnected);
    }

    /// Applies a request after the minimum gap since the last apply, merged with any requests
    /// that queued up meanwhile. The newest caller gets the result; older ones learn theirs was
    /// replaced, so only one of them treats the apply as its own.
    fn apply(&mut self, request: ApplyRequest, reply: Sender<Result<Snapshot, KeyboardError>>, inbox: &Receiver<Job>) {
        if let Some(last) = self.last_apply {
            thread::sleep(MIN_WRITE_GAP.saturating_sub(last.elapsed()));
        }
        let mut request = request;
        let mut replies = vec![reply];
        let mut reads = Vec::new();
        while let Ok(job) = inbox.try_recv() {
            match job {
                Job::Apply(newer, newer_reply) => {
                    request = request.then(newer);
                    replies.push(newer_reply);
                }
                Job::ReadAll(read_reply) => reads.push(read_reply),
            }
        }
        if replies.len() > 1 {
            log::info!("merged {} quick changes into one apply", replies.len());
        }
        let result = self.with_device(|transport, info| session::apply_settings(transport, info, &request));
        self.last_apply = Some(Instant::now());
        // Only the newest caller's change is the one the keyboard ends up holding.
        let newest = replies.pop().expect("at least the first request");
        for reply in replies {
            let _ = reply.send(Err(KeyboardError::Superseded));
        }
        let _ = newest.send(result);
        for read_reply in reads {
            let _ = read_reply.send(self.with_device(session::read_settings));
        }
    }

    /// Runs a job against the open keyboard, dropping the connection if USB itself fails.
    fn with_device(
        &mut self,
        job: impl FnOnce(&mut dyn Transport, DeviceInfo) -> Result<Snapshot, KeyboardError>,
    ) -> Result<Snapshot, KeyboardError> {
        let device = self.device.as_mut().ok_or(KeyboardError::NotConnected)?;
        let result = job(&mut device.transport, device.info.clone());
        if let Err(KeyboardError::Hid(message)) = &result {
            log::warn!("USB error, dropping connection: {message}");
            self.disconnect();
        }
        result
    }

    fn set_status(&self, next: KeyboardStatus) {
        let mut current = self.status.lock().unwrap_or_else(PoisonError::into_inner);
        if *current == next {
            return;
        }
        if let KeyboardStatus::Error { message } = &next {
            log::warn!("keyboard error: {message}");
        }
        *current = next.clone();
        drop(current);
        if let Err(err) = self.app.emit(STATUS_EVENT, next) {
            log::warn!("could not send keyboard status to the window: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_for_the_screens() {
        let json = serde_json::to_value(KeyboardStatus::Unsupported { device_id: 7 }).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "unsupported", "deviceId": 7 }));
        let json = serde_json::to_value(KeyboardStatus::Disconnected).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "disconnected" }));
    }

    fn path(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn plugging_in_connects() {
        let a = path("kb-a");
        assert_eq!(next_step(None, Some(&a), None), Step::Connect);
    }

    #[test]
    fn same_device_stays_connected() {
        let a = path("kb-a");
        assert_eq!(next_step(Some(&a), Some(&a), None), Step::Stay);
    }

    #[test]
    fn unplugging_disconnects() {
        let a = path("kb-a");
        assert_eq!(next_step(Some(&a), None, None), Step::Disconnect);
    }

    #[test]
    fn a_different_device_path_reconnects() {
        let (a, b) = (path("kb-a"), path("kb-b"));
        assert_eq!(next_step(Some(&a), Some(&b), None), Step::Reconnect);
    }

    #[test]
    fn skipped_device_is_left_alone_until_it_changes() {
        let (a, b) = (path("kb-a"), path("kb-b"));
        // Unknown ID or cooling down: don't touch it again.
        assert_eq!(next_step(None, Some(&a), Some(&a)), Step::Stay);
        // A different device appearing is still picked up.
        assert_eq!(next_step(None, Some(&b), Some(&a)), Step::Connect);
    }

    #[test]
    fn nothing_found_reports_missing() {
        assert_eq!(next_step(None, None, None), Step::ReportMissing);
    }

    #[test]
    fn rejected_device_stays_skipped_until_unplugged() {
        let (a, b) = (path("kb-a"), path("kb-b"));
        let now = Instant::now();
        let mut skips = Skips::default();
        skips.reject(a.clone());
        assert_eq!(skips.update(Some(&a), now), Some(a.as_c_str()));
        // Unplugged: forgotten, so plugging it back in tries again.
        assert_eq!(skips.update(None, now), None);
        assert_eq!(skips.update(Some(&a), now), None);
        // Replaced by a different device: also forgotten.
        skips.reject(a.clone());
        assert_eq!(skips.update(Some(&b), now), None);
    }

    #[test]
    fn failed_device_cools_down_then_is_tried_again() {
        let a = path("kb-a");
        let now = Instant::now();
        let mut skips = Skips::default();
        let delay = skips.record_failure(a.clone(), now);
        assert_eq!(delay, Duration::from_secs(2));
        assert_eq!(skips.update(Some(&a), now), Some(a.as_c_str()), "held while cooling");
        assert_eq!(skips.update(Some(&a), now + delay), None, "released when the wait is over");
    }

    #[test]
    fn repeated_failures_wait_longer_and_reset_for_a_new_device() {
        let (a, b) = (path("kb-a"), path("kb-b"));
        let now = Instant::now();
        let mut skips = Skips::default();
        skips.record_failure(a.clone(), now);
        assert_eq!(skips.record_failure(a.clone(), now), Duration::from_secs(5));
        assert_eq!(skips.record_failure(b, now), Duration::from_secs(2), "count is per device");
        // Unplugging forgets the cool-down entirely.
        assert_eq!(skips.update(None, now), None);
        assert_eq!(skips.record_failure(a, now), Duration::from_secs(2));
    }

    #[test]
    fn a_successful_connect_clears_the_cool_down() {
        let a = path("kb-a");
        let now = Instant::now();
        let mut skips = Skips::default();
        skips.record_failure(a.clone(), now);
        skips.clear_failures();
        assert_eq!(skips.update(Some(&a), now), None);
    }

    #[test]
    fn retry_delays_grow_then_cap() {
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(5));
        assert_eq!(retry_delay(4), Duration::from_secs(30));
        assert_eq!(retry_delay(99), Duration::from_secs(30));
    }
}
