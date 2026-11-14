//! Keystra desktop app: sets up logging, opens the profile store, starts the keyboard worker and
//! registers commands.

#[cfg(test)]
mod bindings;
mod commands;
mod keyboard;
mod logging;
mod profiles;

use tauri::Manager;

pub fn run() {
    let result = tauri::Builder::default()
        .plugin(logging::plugin())
        .setup(|app| {
            logging::install_panic_hook();
            log::info!("Keystra {} starting", app.package_info().version);
            let profiles_path = app.path().app_data_dir()?.join(profiles::FILE_NAME);
            app.manage(profiles::ProfilesState::new(profiles::ProfileStore::open(profiles_path)));
            app.manage(keyboard::KeyboardHandle::spawn(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::keyboard_status,
            commands::read_keyboard,
            commands::list_profiles,
            commands::create_profile,
            commands::create_profile_from_keyboard,
            commands::rename_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::save_keyboard_to_profile,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        log::error!("Keystra failed to start: {err}");
        std::process::exit(1);
    }
}
