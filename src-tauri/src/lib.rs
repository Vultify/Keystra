//! Keystra desktop app: sets up logging, opens the profile store, puts Keystra in the tray,
//! starts the keyboard worker and registers commands.

mod apply;
#[cfg(test)]
mod bindings;
mod commands;
mod error;
mod keyboard;
mod logging;
mod profiles;
mod startup;
mod tray;
mod window;

use tauri::Manager;

pub fn run() {
    let result = tauri::Builder::default()
        // Must come first: a second launch hands over to the running Keystra and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| window::show_main(app)))
        .plugin(logging::plugin())
        .on_window_event(window::keep_running_on_close)
        .setup(|app| {
            logging::install_panic_hook();
            log::info!("Keystra {} starting", app.package_info().version);
            let profiles_path = app.path().app_data_dir()?.join(profiles::FILE_NAME);
            app.manage(profiles::ProfilesState::new(profiles::ProfileStore::open(profiles_path)));
            // Both follow the worker's announcements, so they listen before it starts.
            apply::follow_connections(app.handle());
            tray::create(app.handle())?;
            keyboard::KeyboardHandle::start(app.handle());
            if window::started_hidden() {
                window::hide_main(app.handle());
            } else {
                window::show_main(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::keyboard_status,
            commands::read_keyboard,
            commands::list_profiles,
            commands::create_profile,
            commands::create_profile_from_keyboard,
            commands::duplicate_profile,
            commands::rename_profile,
            commands::save_profile,
            commands::delete_profile,
            commands::save_keyboard_to_profile,
            commands::apply_profile,
            commands::start_with_windows,
            commands::set_start_with_windows,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        log::error!("Keystra failed to start: {err}");
        std::process::exit(1);
    }
}
