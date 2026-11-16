//! Start with Windows: a value named after the app under the current user's Run key that opens it
//! hidden in the tray. Only the current user's key is written, even when Keystra runs as
//! administrator, so the setting can always be turned off again from the app.

use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};

use crate::window::STARTED_HIDDEN_ARG;

/// The Run value for this copy of the app. Its name is the product name ("Keystra", or "Keystra
/// Dev" for development builds), which is also the value the installer's uninstaller removes.
fn launcher(product_name: &str) -> auto_launch::Result<AutoLaunch> {
    let exe = std::env::current_exe()?;
    // Quoted, so a path with spaces (like a user folder) launches the right program.
    let path = format!("\"{}\"", exe.display());
    AutoLaunchBuilder::new()
        .set_app_name(product_name)
        .set_app_path(&path)
        .set_args(&[STARTED_HIDDEN_ARG])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .build()
}

/// On when the Run value exists and Task Manager's Startup apps hasn't turned it off. Turning it
/// on always points the value at this exe.
pub fn is_enabled(product_name: &str) -> auto_launch::Result<bool> {
    launcher(product_name)?.is_enabled()
}

/// Turns starting with Windows on or off, then reports what Windows now has.
pub fn set_enabled(product_name: &str, enabled: bool) -> auto_launch::Result<bool> {
    let startup = launcher(product_name)?;
    if enabled {
        startup.enable()?;
    } else {
        startup.disable()?;
    }
    log::info!("start with Windows turned {}", if enabled { "on" } else { "off" });
    startup.is_enabled()
}
