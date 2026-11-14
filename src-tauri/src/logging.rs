//! App logging: a rotating log file in the app's log folder, plus a panic hook so crashes leave a trace.

use log::LevelFilter;
use tauri::Runtime;
use tauri::plugin::TauriPlugin;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

/// Each log file is capped at about 1 MB, and the 3 newest are kept.
const MAX_LOG_BYTES: u128 = 1_000_000;
const KEPT_LOG_FILES: usize = 3;

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    let level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    tauri_plugin_log::Builder::new()
        .clear_targets()
        .target(Target::new(TargetKind::Stdout))
        .target(Target::new(TargetKind::LogDir {
            file_name: Some("keystra".into()),
        }))
        .level(level)
        // The window library logs every redraw at debug level.
        .level_for("tao", LevelFilter::Warn)
        .max_file_size(MAX_LOG_BYTES)
        .rotation_strategy(RotationStrategy::KeepSome(KEPT_LOG_FILES))
        .build()
}

/// Writes panics to the log before the app closes. Release builds abort on panic and have no
/// console, so without this a crash would leave nothing behind.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        default_hook(info);
    }));
}
