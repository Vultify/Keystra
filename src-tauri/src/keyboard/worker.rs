//! The one owner of the keyboard. A single thread holds the USB handle, runs every request in
//! order, notices plugs and unplugs, and tells the screens when the connection changes.

use std::ffi::{CStr, CString};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use hidapi::HidApi;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::conflicts::{self, MagCrateState};
use super::device_events::DeviceWatch;
use super::error::KeyboardError;
use super::pattern::{self, Pattern};
use super::protocol::{Command, PRODUCT_ID, USAGE, USAGE_PAGE, VENDOR_ID};
use super::session::{self, ApplyRequest, Snapshot};
use super::settings::{self, DeviceInfo};
use super::transport::{self, HidTransport, Transport};

/// How often the worker looks for the keyboard when Windows can't announce device changes.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// With device announcements, the worker still looks this often in case one was missed.
const BACKSTOP_INTERVAL: Duration = Duration::from_secs(30);

/// How long to let a burst of device changes settle before looking. A keyboard brings several
/// USB interfaces with it, and they arrive one after another.
const SETTLE_DELAY: Duration = Duration::from_millis(250);

/// While connected, how often MagCrate is looked for.
const MAGCRATE_CHECK_INTERVAL: Duration = Duration::from_secs(3);

/// Waits before retrying a keyboard that failed to answer, growing with each failure.
const RETRY_DELAYS: [Duration; 4] = [
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(10),
    Duration::from_secs(30),
];

/// USB errors this close together count as the same trouble, so their waits keep growing.
const DROP_MEMORY: Duration = Duration::from_secs(60);

/// The least time between two writes to the keyboard (applies or pattern saves), so quick repeated
/// changes can never become a stream of writes. Applies that arrive meanwhile are merged into one.
const MIN_WRITE_GAP: Duration = Duration::from_millis(500);

/// How often a development build checks whether the installed Keystra has quit.
const YIELD_RECHECK: Duration = Duration::from_secs(3);

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
    /// A development build is leaving the keyboard to the installed Keystra, which is running.
    InstalledRunning,
    /// The keyboard kept failing over USB; Keystra tries again after `seconds`.
    Retrying { seconds: u32 },
}

enum Job {
    ReadAll(Sender<Result<Snapshot, KeyboardError>>),
    Apply(ApplyRequest, Sender<Result<Snapshot, KeyboardError>>),
    ReadPattern(u8, Sender<Result<Pattern, KeyboardError>>),
    SavePattern(u8, Pattern, Sender<Result<Pattern, KeyboardError>>),
    /// Windows reported a USB input device arriving or leaving.
    DeviceChanged,
}

type ApplyReply = Sender<Result<Snapshot, KeyboardError>>;

/// An apply with every apply that queued up behind it folded in.
struct Merged {
    request: ApplyRequest,
    /// Callers to answer, oldest first; only the last one gets the result.
    replies: Vec<ApplyReply>,
    /// Other jobs that queued up meanwhile, to run after the apply in order.
    later: Vec<Job>,
}

/// Folds queued applies into the first one (the newest wins wherever it sets a part) and sets
/// every other job aside, keeping its order.
fn merge_applies(first: ApplyRequest, reply: ApplyReply, queued: impl IntoIterator<Item = Job>) -> Merged {
    let mut merged = Merged { request: first, replies: vec![reply], later: Vec::new() };
    for job in queued {
        match job {
            Job::Apply(newer, newer_reply) => {
                merged.request = merged.request.then(newer);
                merged.replies.push(newer_reply);
            }
            other => merged.later.push(other),
        }
    }
    merged
}

/// When the next write to the keyboard may start: at least MIN_WRITE_GAP after the last one.
#[derive(Default)]
struct WriteGap {
    last: Option<Instant>,
}

impl WriteGap {
    fn wait(&self, now: Instant) -> Duration {
        self.last.map_or(Duration::ZERO, |last| MIN_WRITE_GAP.saturating_sub(now.saturating_duration_since(last)))
    }

    fn wrote(&mut self, at: Instant) {
        self.last = Some(at);
    }
}

/// Cheap handle the commands use to talk to the worker thread.
#[derive(Clone)]
pub struct KeyboardHandle {
    jobs: Sender<Job>,
    status: Arc<Mutex<KeyboardStatus>>,
}

impl KeyboardHandle {
    /// Registers the handle with the app, then starts the worker thread. In that order, anything
    /// reacting to the worker's first announcement can always find the handle.
    pub fn start(app: &AppHandle) {
        let (jobs, inbox) = mpsc::channel();
        let status = Arc::new(Mutex::new(KeyboardStatus::Disconnected));
        let changes = jobs.clone();
        let watch = DeviceWatch::hid_changes(move || {
            let _ = changes.send(Job::DeviceChanged);
        });
        let worker = Worker {
            app: app.clone(),
            status: Arc::clone(&status),
            api: None,
            device: None,
            skips: Skips::default(),
            magcrate: MagCrateState::Closed,
            schedule: Schedule::new(watch.is_some(), Instant::now()),
            _watch: watch,
            gap: WriteGap::default(),
            yielded: false,
        };
        app.manage(Self { jobs, status });
        thread::Builder::new()
            .name("keyboard-worker".into())
            .spawn(move || worker.run(inbox))
            .expect("failed to start the keyboard worker thread");
    }

    pub fn status(&self) -> KeyboardStatus {
        self.status.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status(), KeyboardStatus::Connected { .. })
    }

    /// Reads every setting Keystra knows in one go. Blocks until the worker answers.
    pub fn read_all(&self) -> Result<Snapshot, KeyboardError> {
        self.ask(Job::ReadAll)
    }

    /// Writes whatever in `request` differs from the keyboard, confirming each write with a read.
    /// Blocks until the worker answers with the keyboard's settings afterwards.
    pub fn apply(&self, request: ApplyRequest) -> Result<Snapshot, KeyboardError> {
        self.ask(|reply| Job::Apply(request, reply))
    }

    /// Reads one of the keyboard's saved per-key color patterns.
    pub fn read_pattern(&self, slot: u8) -> Result<Pattern, KeyboardError> {
        self.ask(|reply| Job::ReadPattern(slot, reply))
    }

    /// Saves a per-key color pattern to the keyboard and returns what it now holds.
    pub fn save_pattern(&self, slot: u8, pattern: Pattern) -> Result<Pattern, KeyboardError> {
        self.ask(|reply| Job::SavePattern(slot, pattern, reply))
    }

    /// Sends a job to the worker and waits for its answer.
    fn ask<T>(&self, job: impl FnOnce(Sender<Result<T, KeyboardError>>) -> Job) -> Result<T, KeyboardError> {
        let (reply, answer) = mpsc::channel();
        self.jobs.send(job(reply)).map_err(|_| KeyboardError::WorkerStopped)?;
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
    /// USB errors on a connected keyboard, each within DROP_MEMORY of the one before. Survives
    /// reconnects on purpose, so a job that keeps failing can't loop.
    drops: usize,
    last_drop: Option<Instant>,
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

    /// Records a USB error on a connected keyboard. The first one reconnects right away (often a
    /// glitch); repeats soon after wait like failed connects, and a failed connect keeps escalating.
    fn record_drop(&mut self, path: CString, now: Instant) -> Option<Duration> {
        let recent = self.last_drop.is_some_and(|last| now.saturating_duration_since(last) < DROP_MEMORY);
        self.drops = if recent { self.drops + 1 } else { 1 };
        self.last_drop = Some(now);
        if self.drops < 2 {
            return None;
        }
        let failures = self.drops - 1;
        let delay = retry_delay(failures);
        self.cooldown = Some(Cooldown { path, retry_at: now + delay, failures });
        Some(delay)
    }

    /// When a device that's cooling down may be tried again, if that's still ahead.
    fn retry_at(&self, now: Instant) -> Option<Instant> {
        self.cooldown.as_ref().map(|c| c.retry_at).filter(|&at| at > now)
    }
}

/// When the worker next looks for the keyboard and for MagCrate. Only timing, so it's tested
/// without USB.
#[derive(Debug)]
struct Schedule {
    /// Whether Windows announces device changes. Without them the worker looks every second.
    watching: bool,
    look_at: Instant,
    magcrate_at: Instant,
}

impl Schedule {
    fn new(watching: bool, now: Instant) -> Self {
        Self {
            watching,
            look_at: now,
            magcrate_at: now + MAGCRATE_CHECK_INTERVAL,
        }
    }

    /// Plans the next look after one just happened, sooner when a cooling device becomes due.
    fn looked(&mut self, now: Instant, retry_at: Option<Instant>) {
        let regular = now + if self.watching { BACKSTOP_INTERVAL } else { POLL_INTERVAL };
        self.look_at = retry_at.map_or(regular, |at| at.min(regular));
    }

    /// A device arrived or left, so look once the burst of changes settles.
    fn device_changed(&mut self, now: Instant) {
        self.look_at = self.look_at.min(now + SETTLE_DELAY);
    }

    fn magcrate_checked(&mut self, now: Instant) {
        self.magcrate_at = now + MAGCRATE_CHECK_INTERVAL;
    }

    fn look_due(&self, now: Instant) -> bool {
        now >= self.look_at
    }

    fn magcrate_due(&self, now: Instant) -> bool {
        now >= self.magcrate_at
    }

    /// How long to wait for a job before the next timed check. MagCrate only matters while connected.
    fn wait(&self, now: Instant, connected: bool) -> Duration {
        let next = if connected { self.look_at.min(self.magcrate_at) } else { self.look_at };
        next.saturating_duration_since(now)
    }
}

struct Worker {
    app: AppHandle,
    status: Arc<Mutex<KeyboardStatus>>,
    api: Option<HidApi>,
    device: Option<Connected>,
    skips: Skips,
    magcrate: MagCrateState,
    schedule: Schedule,
    /// Keeps Windows' device announcements coming for as long as the worker runs.
    _watch: Option<DeviceWatch>,
    gap: WriteGap,
    /// Set while a development build leaves the keyboard to the installed Keystra.
    yielded: bool,
}

impl Worker {
    fn run(mut self, inbox: Receiver<Job>) {
        log::info!("keyboard worker started");
        self.refresh();
        loop {
            let wait = self.schedule.wait(Instant::now(), self.device.is_some());
            match inbox.recv_timeout(wait) {
                Ok(job) => self.handle(job, &inbox),
                Err(RecvTimeoutError::Timeout) => self.on_timer(),
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        log::info!("keyboard worker stopped");
    }

    // A caller may have gone away before its answer is sent; nothing to do if so.
    fn handle(&mut self, job: Job, inbox: &Receiver<Job>) {
        match job {
            Job::ReadAll(reply) => {
                let _ = reply.send(self.with_device(session::read_settings));
            }
            Job::Apply(request, reply) => self.apply(request, reply, inbox),
            Job::ReadPattern(slot, reply) => {
                let _ = reply.send(self.with_device(|transport, _| pattern::read_pattern(transport, slot)));
            }
            Job::SavePattern(slot, colors, reply) => {
                let result = self.write(|transport, _| pattern::write_pattern(transport, slot, &colors));
                let _ = reply.send(result);
            }
            Job::DeviceChanged => self.schedule.device_changed(Instant::now()),
        }
    }

    fn wait_for_write_gap(&self) {
        thread::sleep(self.gap.wait(Instant::now()));
    }

    /// Runs a job that writes to the keyboard, at least MIN_WRITE_GAP after the last write. Every
    /// write job goes through here, so none can skip the gap.
    fn write<T>(
        &mut self,
        job: impl FnOnce(&mut dyn Transport, DeviceInfo) -> Result<T, KeyboardError>,
    ) -> Result<T, KeyboardError> {
        self.wait_for_write_gap();
        let connected = self.device.is_some();
        let result = self.with_device(job);
        if connected {
            self.gap.wrote(Instant::now());
        }
        result
    }

    fn on_timer(&mut self) {
        if self.schedule.look_due(Instant::now()) {
            self.refresh();
        }
        if self.device.is_some() && self.schedule.magcrate_due(Instant::now()) {
            self.check_conflicts();
        }
    }

    /// Notices plugs and unplugs, and connects to the keyboard when it appears (checking for MagCrate as it does).
    fn refresh(&mut self) {
        let found = self.find_keyboard();
        let found = match found {
            Ok(path) => path,
            Err(err) => {
                self.schedule.looked(Instant::now(), None);
                self.set_status(KeyboardStatus::Error { message: err.to_string() });
                return;
            }
        };

        let skipped = self.skips.update(found.as_deref(), Instant::now());
        let open = self.device.as_ref().map(|d| d.path.as_c_str());
        match next_step(open, found.as_deref(), skipped) {
            Step::Stay => {}
            Step::Connect => {
                self.check_conflicts();
                if !self.yields_to_installed() {
                    self.connect(found.expect("Connect means a device was found"));
                }
            }
            Step::Reconnect => {
                self.release_device();
                self.check_conflicts();
                if !self.yields_to_installed() {
                    self.connect(found.expect("Reconnect means a device was found"));
                }
            }
            Step::Disconnect => self.disconnect(),
            Step::ReportMissing => self.set_status(KeyboardStatus::Disconnected),
        }
        let now = Instant::now();
        let retry_at = if self.yielded { Some(now + YIELD_RECHECK) } else { self.skips.retry_at(now) };
        self.schedule.looked(now, retry_at);
    }

    /// A development build leaves the keyboard to an installed Keystra that's running, so the two
    /// never fight over it. Release builds always connect.
    fn yields_to_installed(&mut self) -> bool {
        self.yielded = cfg!(debug_assertions) && conflicts::installed_keystra_running();
        if self.yielded {
            self.release_device();
            self.set_status(KeyboardStatus::InstalledRunning);
            // Check again soon, so the keyboard comes back quickly once the installed app quits.
            let now = Instant::now();
            self.schedule.looked(now, Some(now + YIELD_RECHECK));
        }
        self.yielded
    }

    fn check_conflicts(&mut self) {
        if self.device.is_some() && self.yields_to_installed() {
            return;
        }
        let state = conflicts::magcrate_state();
        self.schedule.magcrate_checked(Instant::now());
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

    /// Lets go of the keyboard without announcing a status; the caller sets the one that fits.
    fn release_device(&mut self) {
        if self.device.take().is_some() {
            log::info!("keyboard disconnected");
        }
    }

    fn disconnect(&mut self) {
        self.release_device();
        self.set_status(KeyboardStatus::Disconnected);
    }

    /// Applies a request after the minimum gap since the last write, merged with any requests
    /// that queued up meanwhile. The newest caller gets the result; older ones learn theirs was
    /// replaced, so only one of them treats the apply as its own.
    fn apply(&mut self, request: ApplyRequest, reply: ApplyReply, inbox: &Receiver<Job>) {
        // Waiting first lets requests made during the gap join this apply.
        self.wait_for_write_gap();
        let Merged { request, mut replies, later } = merge_applies(request, reply, std::iter::from_fn(|| inbox.try_recv().ok()));
        if replies.len() > 1 {
            log::info!("merged {} quick changes into one apply", replies.len());
        }
        let result = self.write(|transport, info| session::apply_settings(transport, info, &request));
        // Only the newest caller's change is the one the keyboard ends up holding.
        let newest = replies.pop().expect("at least the first request");
        for reply in replies {
            let _ = reply.send(Err(KeyboardError::Superseded));
        }
        let _ = newest.send(result);
        for job in later {
            self.handle(job, inbox);
        }
    }

    /// Runs a job against the open keyboard, dropping the connection if USB itself fails.
    fn with_device<T>(
        &mut self,
        job: impl FnOnce(&mut dyn Transport, DeviceInfo) -> Result<T, KeyboardError>,
    ) -> Result<T, KeyboardError> {
        let device = self.device.as_mut().ok_or(KeyboardError::NotConnected)?;
        let result = job(&mut device.transport, device.info.clone());
        if let Err(KeyboardError::Hid(message)) = &result {
            log::warn!("USB error, dropping connection: {message}");
            let path = device.path.clone();
            self.release_device();
            match self.skips.record_drop(path, Instant::now()) {
                Some(delay) => {
                    log::warn!("the keyboard keeps failing, waiting {}s before trying again", delay.as_secs());
                    self.set_status(KeyboardStatus::Retrying { seconds: u32::try_from(delay.as_secs()).unwrap_or(u32::MAX) });
                }
                None => self.set_status(KeyboardStatus::Disconnected),
            }
            // Windows won't announce anything if the keyboard is still plugged in, so look again
            // soon. That look plans the next one for when any cooldown ends.
            self.schedule.device_changed(Instant::now());
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
    fn writes_keep_half_a_second_apart() {
        let start = Instant::now();
        let mut gap = WriteGap::default();
        assert_eq!(gap.wait(start), Duration::ZERO, "the first write never waits");
        gap.wrote(start);
        assert_eq!(gap.wait(start + Duration::from_millis(100)), Duration::from_millis(400));
        assert_eq!(gap.wait(start + Duration::from_millis(900)), Duration::ZERO);
    }

    #[test]
    fn queued_applies_become_one_and_other_jobs_keep_their_order() {
        let (first_reply, first_answer) = mpsc::channel();
        let (second_reply, _) = mpsc::channel();
        let (third_reply, _) = mpsc::channel();
        let (read_reply, _) = mpsc::channel();
        let queued = vec![
            Job::Apply(ApplyRequest { polling_rate_hz: Some(8000), ..ApplyRequest::default() }, second_reply),
            Job::ReadAll(read_reply),
            Job::DeviceChanged,
            Job::Apply(ApplyRequest { onboard_slot: Some(2), ..ApplyRequest::default() }, third_reply),
        ];
        let first = ApplyRequest { onboard_slot: Some(1), polling_rate_hz: Some(1000), lighting: None };
        let merged = merge_applies(first, first_reply, queued);
        assert_eq!(merged.request.onboard_slot, Some(2));
        assert_eq!(merged.request.polling_rate_hz, Some(8000));
        assert_eq!(merged.replies.len(), 3);
        assert!(matches!(merged.later.as_slice(), [Job::ReadAll(_), Job::DeviceChanged]));
        // The oldest caller is answered first, so only the newest one is left holding the result.
        merged.replies[0].send(Err(KeyboardError::Superseded)).unwrap();
        assert!(matches!(first_answer.recv().unwrap(), Err(KeyboardError::Superseded)));
    }

    #[test]
    fn a_keyboard_that_keeps_dropping_backs_off() {
        let path = CString::new("kb").unwrap();
        let now = Instant::now();
        let mut skips = Skips::default();
        assert_eq!(skips.record_drop(path.clone(), now), None, "a single glitch reconnects at once");
        skips.clear_failures(); // the reconnect succeeds, but the drop still counts
        assert_eq!(skips.record_drop(path.clone(), now), Some(retry_delay(1)));
        assert_eq!(skips.update(Some(&path), now), Some(path.as_c_str()));
        // A failed connect after the wait keeps escalating instead of starting over.
        assert_eq!(skips.record_failure(path.clone(), now), retry_delay(2));
        let later = now + DROP_MEMORY;
        assert_eq!(skips.record_drop(path, later), None, "a drop long after the last starts the count over");
    }

    #[test]
    fn status_serializes_for_the_screens() {
        let json = serde_json::to_value(KeyboardStatus::Unsupported { device_id: 7 }).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "unsupported", "deviceId": 7 }));
        let json = serde_json::to_value(KeyboardStatus::Disconnected).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "disconnected" }));
        let info = DeviceInfo { device_id: 2494, firmware_version: 0x0321, model: Some("MX 8.2 Pro TMR") };
        let json = serde_json::to_value(KeyboardStatus::Connected { info, magcrate: MagCrateState::Closed }).unwrap();
        assert_eq!(json["state"], "connected");
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
    fn with_device_announcements_the_worker_mostly_waits() {
        let now = Instant::now();
        let mut schedule = Schedule::new(true, now);
        schedule.looked(now, None);
        assert_eq!(schedule.wait(now, false), BACKSTOP_INTERVAL);
        assert_eq!(schedule.wait(now, true), MAGCRATE_CHECK_INTERVAL, "MagCrate is checked while connected");
    }

    #[test]
    fn without_device_announcements_the_worker_looks_every_second() {
        let now = Instant::now();
        let mut schedule = Schedule::new(false, now);
        schedule.looked(now, None);
        assert_eq!(schedule.wait(now, false), POLL_INTERVAL);
    }

    #[test]
    fn a_device_change_brings_the_next_look_forward_once() {
        let now = Instant::now();
        let mut schedule = Schedule::new(true, now);
        schedule.looked(now, None);
        schedule.device_changed(now);
        assert_eq!(schedule.wait(now, false), SETTLE_DELAY);
        // More changes in the same burst don't keep pushing the look back.
        schedule.device_changed(now + Duration::from_millis(200));
        assert_eq!(schedule.wait(now, false), SETTLE_DELAY);
        assert!(schedule.look_due(now + SETTLE_DELAY));
    }

    #[test]
    fn a_cooling_device_is_tried_again_when_due() {
        let now = Instant::now();
        let mut schedule = Schedule::new(true, now);
        schedule.looked(now, Some(now + Duration::from_secs(5)));
        assert_eq!(schedule.wait(now, false), Duration::from_secs(5));
        // A retry further off than the backstop doesn't delay the regular look.
        schedule.looked(now, Some(now + Duration::from_secs(90)));
        assert_eq!(schedule.wait(now, false), BACKSTOP_INTERVAL);
    }

    #[test]
    fn only_a_future_retry_counts() {
        let now = Instant::now();
        let mut skips = Skips::default();
        skips.record_failure(path("kb"), now);
        assert_eq!(skips.retry_at(now), Some(now + Duration::from_secs(2)));
        assert_eq!(skips.retry_at(now + Duration::from_secs(3)), None);
    }

    #[test]
    fn retry_delays_grow_then_cap() {
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(5));
        assert_eq!(retry_delay(4), Duration::from_secs(30));
        assert_eq!(retry_delay(99), Duration::from_secs(30));
    }
}
