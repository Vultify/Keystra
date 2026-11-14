//! Tauri commands the screens call. Each one hands its work to the keyboard worker.

use tauri::State;

use crate::keyboard::{KeyboardError, KeyboardHandle, KeyboardStatus, Snapshot};

#[tauri::command]
pub fn keyboard_status(keyboard: State<'_, KeyboardHandle>) -> KeyboardStatus {
    keyboard.status()
}

/// Reads the keyboard's current settings. Runs off the main thread so the window stays responsive.
#[tauri::command]
pub async fn read_keyboard(keyboard: State<'_, KeyboardHandle>) -> Result<Snapshot, KeyboardError> {
    let keyboard = keyboard.inner().clone();
    tauri::async_runtime::spawn_blocking(move || keyboard.read_all())
        .await
        .map_err(|_| KeyboardError::WorkerStopped)?
}
