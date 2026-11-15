//! The tray icon: switch profiles, open the window or quit, all with the window closed. The menu
//! is rebuilt whenever the profiles or the connection change.

use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Listener, Manager, Wry};

use crate::apply;
use crate::keyboard::STATUS_EVENT;
use crate::profiles::{PROFILES_EVENT, ProfileList, ProfilesState};
use crate::window;

const TRAY_ID: &str = "keystra";
const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";
const PROFILE_ID_PREFIX: &str = "profile:";

/// What the tray shows, worked out from the profiles and the connection. No tray needed, so it's tested.
#[derive(Debug, PartialEq)]
struct TrayModel {
    tooltip: String,
    /// Why the profiles can't be picked right now, shown above them.
    note: Option<&'static str>,
    profiles: Vec<TrayProfile>,
}

#[derive(Debug, PartialEq)]
struct TrayProfile {
    id: u32,
    /// Menu text: Windows reads "&" as a shortcut marker, so a real one is doubled.
    label: String,
    active: bool,
}

fn model(list: &ProfileList, connected: bool) -> TrayModel {
    let note = if list.read_only.is_some() {
        Some("Profiles are read-only")
    } else if list.active_id.is_none() {
        // First run copies the keyboard's own settings into a profile; a tray switch would skip that.
        Some("Open Keystra to finish setup")
    } else if !connected {
        Some("Keyboard not connected")
    } else {
        None
    };
    let unsent = list.unsent_id.filter(|&id| Some(id) == list.active_id);
    let active = list.profiles.iter().find(|profile| Some(profile.id) == list.active_id);
    let tooltip = match active {
        Some(profile) if unsent.is_some() => format!("Keystra: {} didn't reach the keyboard. Pick it again to retry.", profile.name),
        Some(profile) => format!("Keystra: {}", profile.name),
        None => "Keystra".to_string(),
    };
    TrayModel {
        tooltip,
        note,
        profiles: list
            .profiles
            .iter()
            .map(|profile| {
                let name = profile.name.replace('&', "&&");
                TrayProfile {
                    id: profile.id,
                    label: if unsent == Some(profile.id) { format!("{name} (try again)") } else { name },
                    active: Some(profile.id) == list.active_id,
                }
            })
            .collect(),
    }
}

/// Puts the icon in the tray and keeps its menu current.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let model = current_model(app);
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(&model.tooltip)
        .menu(&build_menu(app, &model)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                window::show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    for event in [PROFILES_EVENT, STATUS_EVENT] {
        let handle = app.clone();
        app.listen(event, move |_| refresh_soon(&handle));
    }
    Ok(())
}

/// Rebuilds the menu on the main thread, which owns it. Events can arrive from any thread.
fn refresh_soon(app: &AppHandle) {
    let handle = app.clone();
    if let Err(err) = app.run_on_main_thread(move || refresh(&handle)) {
        log::warn!("couldn't update the tray menu: {err}");
    }
}

fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let model = current_model(app);
    let updated = build_menu(app, &model)
        .and_then(|menu| tray.set_menu(Some(menu)))
        .and_then(|()| tray.set_tooltip(Some(&model.tooltip)));
    if let Err(err) = updated {
        log::warn!("couldn't update the tray menu: {err}");
    }
}

fn current_model(app: &AppHandle) -> TrayModel {
    model(&app.state::<ProfilesState>().list(), apply::keyboard_connected(app))
}

fn build_menu(app: &AppHandle, model: &TrayModel) -> tauri::Result<Menu<Wry>> {
    let mut items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    if let Some(note) = model.note {
        items.push(Box::new(MenuItem::with_id(app, "note", note, false, None::<&str>)?));
    }
    for profile in &model.profiles {
        let id = format!("{PROFILE_ID_PREFIX}{}", profile.id);
        let enabled = model.note.is_none();
        items.push(Box::new(CheckMenuItem::with_id(app, id, &profile.label, enabled, profile.active, None::<&str>)?));
    }
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(app, OPEN_ID, "Open Keystra", true, None::<&str>)?));
    items.push(Box::new(MenuItem::with_id(app, QUIT_ID, "Quit Keystra", true, None::<&str>)?));
    let items: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &items)
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        OPEN_ID => window::show_main(app),
        QUIT_ID => app.exit(0),
        _ => {
            if let Some(profile_id) = id.strip_prefix(PROFILE_ID_PREFIX).and_then(|rest| rest.parse().ok()) {
                switch_to(app, profile_id);
            }
        }
    }
}

/// Switches profile off the main thread, since it waits for the keyboard. Picking the profile
/// already in use sends it again, which is how a failed switch is retried; only differences are written.
fn switch_to(app: &AppHandle, id: u32) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(err) = apply::apply_profile(&app, id) {
            log::warn!("tray switch to profile {id} failed: {err}");
        }
        // Clicking a check item flips its tick, so redraw even when nothing changed.
        refresh_soon(&app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::Profile;

    fn list(active_id: Option<u32>) -> ProfileList {
        ProfileList {
            profiles: vec![Profile::factory(1, "Default"), Profile::factory(2, "Games & stuff")],
            active_id,
            read_only: None,
            unsent_id: None,
        }
    }

    #[test]
    fn shows_every_profile_with_the_active_one_ticked() {
        let tray = model(&list(Some(2)), true);
        assert_eq!(tray.tooltip, "Keystra: Games & stuff");
        assert_eq!(tray.note, None);
        assert_eq!(
            tray.profiles,
            vec![
                TrayProfile { id: 1, label: "Default".into(), active: false },
                TrayProfile { id: 2, label: "Games && stuff".into(), active: true },
            ]
        );
    }

    #[test]
    fn says_why_profiles_cant_be_picked() {
        assert_eq!(model(&list(Some(1)), false).note, Some("Keyboard not connected"));
        let read_only = ProfileList { read_only: Some("bad file".into()), ..list(Some(1)) };
        assert_eq!(model(&read_only, true).note, Some("Profiles are read-only"));
    }

    #[test]
    fn profiles_stay_locked_until_first_run_is_done() {
        let tray = model(&list(None), true);
        assert_eq!(tray.note, Some("Open Keystra to finish setup"));
        assert_eq!(tray.tooltip, "Keystra");
    }

    #[test]
    fn a_profile_that_didnt_reach_the_keyboard_offers_a_retry() {
        let tray = model(&ProfileList { unsent_id: Some(2), ..list(Some(2)) }, true);
        assert_eq!(tray.profiles[1].label, "Games && stuff (try again)");
        assert!(tray.tooltip.contains("didn't reach the keyboard"));
        // A failure left over from a profile that's no longer in use isn't shown.
        let tray = model(&ProfileList { unsent_id: Some(1), ..list(Some(2)) }, true);
        assert_eq!(tray.profiles[0].label, "Default");
        assert_eq!(tray.tooltip, "Keystra: Games & stuff");
    }
}
