//! Start with Windows: a value named after the app under the current user's Run key that opens it
//! hidden in the tray. Only the current user's key is written, even when Keystra runs as
//! administrator, so the setting can always be turned off again from the app.

use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use windows_sys::Win32::Foundation::ERROR_SUCCESS;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};

use crate::window::STARTED_HIDDEN_ARG;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// The Run value for this copy of the app. Its name is the product name ("Keystra", or "Keystra
/// Dev" for development builds), which is also the value the installer's uninstaller removes.
struct Startup {
    name: String,
    command: String,
    launcher: AutoLaunch,
}

impl Startup {
    fn new(product_name: &str) -> auto_launch::Result<Self> {
        let exe = std::env::current_exe()?;
        // Quoted, so a path with spaces (like a user folder) launches the right program.
        let path = format!("\"{}\"", exe.display());
        let launcher = AutoLaunchBuilder::new()
            .set_app_name(product_name)
            .set_app_path(&path)
            .set_args(&[STARTED_HIDDEN_ARG])
            .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
            .build()?;
        Ok(Self { name: product_name.to_string(), command: format!("{path} {STARTED_HIDDEN_ARG}"), launcher })
    }

    /// On only when the value starts this exe (not another copy, like an old dev build) and Windows
    /// hasn't turned it off in Task Manager's Startup apps.
    fn is_enabled(&self) -> bool {
        let ours = run_value(&self.name).is_some_and(|command| command.eq_ignore_ascii_case(&self.command));
        ours && self.launcher.is_enabled().unwrap_or(false)
    }
}

pub fn is_enabled(product_name: &str) -> auto_launch::Result<bool> {
    Ok(Startup::new(product_name)?.is_enabled())
}

/// Turns starting with Windows on or off, then reports what Windows now has.
pub fn set_enabled(product_name: &str, enabled: bool) -> auto_launch::Result<bool> {
    let startup = Startup::new(product_name)?;
    if enabled {
        startup.launcher.enable()?;
    } else {
        startup.launcher.disable()?;
    }
    log::info!("start with Windows turned {}", if enabled { "on" } else { "off" });
    Ok(startup.is_enabled())
}

/// The command stored under the current user's Run key for `name`, if there is one.
fn run_value(name: &str) -> Option<String> {
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let (key, value) = (wide(RUN_KEY), wide(name));
    let mut buffer = vec![0u16; 1024];
    let mut size = (buffer.len() * 2) as u32;
    // SAFETY: both names are null-terminated and the buffer is `size` bytes long.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let chars = (size as usize / 2).saturating_sub(1);
    Some(String::from_utf16_lossy(&buffer[..chars]))
}
