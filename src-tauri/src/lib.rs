// Entry point for the Keystra desktop app. Keyboard commands get registered here in Phase 2.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Keystra");
}
