//! Start with Windows: a value named Keystra under the current user's Run key that opens Keystra
//! hidden in the tray. Only the current user's key is written, even when Keystra runs as
//! administrator, so the setting can always be turned off again from the app.

use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};

use crate::window::STARTED_HIDDEN_ARG;

/// The Run value's name. The installer's uninstaller removes a value with the product name.
const RUN_VALUE_NAME: &str = "Keystra";

fn launcher() -> auto_launch::Result<AutoLaunch> {
    let exe = std::env::current_exe()?;
    // Quoted, so a path with spaces (like a user folder) launches the right program.
    let path = format!("\"{}\"", exe.display());
    AutoLaunchBuilder::new()
        .set_app_name(RUN_VALUE_NAME)
        .set_app_path(&path)
        .set_args(&[STARTED_HIDDEN_ARG])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .build()
}

pub fn is_enabled() -> auto_launch::Result<bool> {
    launcher()?.is_enabled()
}

/// Turns starting with Windows on or off, then reports what Windows now has.
pub fn set_enabled(enabled: bool) -> auto_launch::Result<bool> {
    let startup = launcher()?;
    if enabled {
        startup.enable()?;
    } else {
        startup.disable()?;
    }
    log::info!("start with Windows turned {}", if enabled { "on" } else { "off" });
    startup.is_enabled()
}
