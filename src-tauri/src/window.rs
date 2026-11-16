//! The main window's life: it's only created when first shown (so a sign-in start in the tray loads
//! no page), hides to the tray instead of closing, and pauses its page while minimized.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, Webview, WebviewWindow, WebviewWindowBuilder, Window, WindowEvent};

const MAIN_WINDOW: &str = "main";

/// Added to the command line when Windows starts Keystra at sign-in, so it opens in the tray.
pub const STARTED_HIDDEN_ARG: &str = "--hidden";

pub fn started_hidden() -> bool {
    std::env::args().any(|arg| arg == STARTED_HIDDEN_ARG)
}

/// Set while the main window is being created, so two quick requests don't both build it.
static CREATING: AtomicBool = AtomicBool::new(false);

/// Shows the main window and brings it to the front, creating it the first time. Creating happens
/// on its own thread: building a webview from inside a tray or launch event can deadlock on Windows.
pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        show(&window);
        return;
    }
    if CREATING.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        if let Some(window) = create_main(&app) {
            show(&window);
        }
        CREATING.store(false, Ordering::SeqCst);
    });
}

fn show(window: &WebviewWindow) {
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

/// Builds the main window from its config entry (`create: false`), titled with the product name.
fn create_main(app: &AppHandle) -> Option<WebviewWindow> {
    let config = app.config().app.windows.iter().find(|w| w.label == MAIN_WINDOW)?;
    let built = WebviewWindowBuilder::from_config(app, config).and_then(|builder| builder.title(&app.package_info().name).build());
    match built {
        Ok(window) => Some(window),
        Err(err) => {
            log::error!("couldn't create the window: {err}");
            None
        }
    }
}

/// Hides the main window to the tray, if it has been opened.
pub fn hide_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        hide(&window);
    }
}

/// Set while the webview is hidden because the window is minimized.
static MINIMIZED: AtomicBool = AtomicBool::new(false);

/// Closing the window hides it, so Keystra keeps running in the tray (Quit lives in the tray menu).
/// Minimizing hides the webview too, so the page stops drawing until the window comes back.
pub fn on_window_event(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            hide_main(window.app_handle());
        }
        WindowEvent::Resized(_) => follow_minimize(window),
        _ => {}
    }
}

fn follow_minimize(window: &Window) {
    let Some(main) = window.app_handle().get_webview_window(MAIN_WINDOW) else { return };
    let webview: &Webview = main.as_ref();
    let minimized = main.is_minimized().unwrap_or(false);
    let result = if minimized && !MINIMIZED.swap(true, Ordering::SeqCst) {
        webview.hide()
    } else if !minimized && MINIMIZED.swap(false, Ordering::SeqCst) {
        webview.show()
    } else {
        Ok(())
    };
    if let Err(err) = result {
        log::warn!("couldn't follow the window being minimized or restored: {err}");
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
