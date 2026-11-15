//! The main window's life: it hides to the tray instead of closing, starts hidden when Windows
//! launches Keystra at sign-in, and comes back to the front when asked.

use tauri::{AppHandle, Manager, Webview, WebviewWindow, Window, WindowEvent};

const MAIN_WINDOW: &str = "main";

/// Added to the command line when Windows starts Keystra at sign-in, so it opens in the tray.
pub const STARTED_HIDDEN_ARG: &str = "--hidden";

pub fn started_hidden() -> bool {
    std::env::args().any(|arg| arg == STARTED_HIDDEN_ARG)
}

/// Shows the main window and brings it to the front, even when it was hidden or minimized.
pub fn show_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else { return };
    let webview: &Webview = window.as_ref();
    let shown = window
        .unminimize()
        .and_then(|()| window.show())
        .and_then(|()| webview.show())
        .and_then(|()| window.set_focus());
    if let Err(err) = shown {
        log::warn!("couldn't show the window: {err}");
    }
}

/// Hides the main window to the tray.
pub fn hide_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        hide(&window);
    }
}

/// Closing the window hides it, so Keystra keeps running in the tray. Quit lives in the tray menu.
pub fn keep_running_on_close(window: &Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        hide_main(window.app_handle());
    }
}

/// Hiding only the window leaves the page thinking it's on screen, so it keeps animating. Hiding
/// the webview too tells it to pause.
fn hide(window: &WebviewWindow) {
    let webview: &Webview = window.as_ref();
    if let Err(err) = webview.hide().and_then(|()| window.hide()) {
        log::warn!("couldn't hide the window: {err}");
    }
}
