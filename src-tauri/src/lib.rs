//! Keystra desktop app: sets up logging, starts the keyboard worker and registers commands.

mod commands;
mod keyboard;
mod logging;

use tauri::Manager;

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(logging::plugin())
        .setup(|app| {
            logging::install_panic_hook();
            log::info!("Keystra {} starting", app.package_info().version);
            app.manage(keyboard::KeyboardHandle::spawn(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::keyboard_status,
            commands::read_keyboard
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        log::error!("Keystra failed to start: {err}");
        std::process::exit(1);
    }
}
