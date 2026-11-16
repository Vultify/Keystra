# Changelog

All notable changes to Keystra are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-16

The first release.

### Added

- Tauri desktop app with a React and TypeScript interface.
- Dark theme built around the Keystra logo purple, with Exo 2 headings and Geist body text.
- App icon made from the Keystra K mark.
- App shell with a sidebar.
- Protocol notes for the keyboard in `research/PROTOCOL-NOTES.md`.
- Node 24 and Rust 1.90 minimum versions pinned in the project.
- Keyboard connection over USB: Keystra finds the MX 8.2 Pro TMR, checks its device ID, and follows plugging and unplugging.
- Read-only access to the firmware version, active profile, polling rate and lighting settings.
- Diagnostics page that shows the keyboard's current settings and the raw bytes behind them.
- Command allowlist, with firmware update, bootloader, reset and calibration commands blocked in code.
- Crash logging to a rotating log file, and a recovery screen if the window crashes.
- Warning when MagCrate is running, since both apps share the keyboard's reply channel, plus retries that recover when the two apps' replies get mixed up. A softer note appears when only MagCrate's leftover background helper is running.
- Lighting mode names match MagCrate's, and Music Sync's rainbow and color settings read correctly.
- The never-send list is checked again on the exact bytes at the moment they're sent.
- A keyboard that fails to answer is retried after a growing wait instead of every second.
- The screens' TypeScript types and shared constants are generated from the Rust code, so the two sides can't drift apart.
- Saved profiles: lighting, polling rate, onboard profile and linked games, stored safely on this PC with a versioned format.
- A profiles file that isn't valid JSON is kept as a backup and replaced. Any other problem with it leaves the file untouched and shows profiles as read-only. Saves are flushed to disk before they replace the old file.
- Applying a profile sends its onboard profile, polling rate and lighting to the keyboard. Keystra writes only what differs, confirms each write by reading it back, and remembers the active profile, shown as "Name (Profile N)" on Diagnostics.
- Quick repeated changes are merged into one write, at most one every half second, so the keyboard is never written in a stream.
- Lighting, Performance and Profiles pages. Every lighting effect is a tile with a small keyboard that plays it when you hover it, next to an Adjust panel with 5-step brightness and speed, Solid or Rainbow color with round swatches and a pick-any-color button, and direction buttons. A pick reaches the keyboard at once. Profiles can be switched, created, renamed, duplicated and deleted.
- A welcome screen on first run that finds the keyboard and starts the first profile from its current settings.
- One clear notice on every page when the keyboard is missing, busy, or MagCrate is open.
- After a failed apply, the screen shows what the keyboard really holds. Saving a profile that reached the file but not the keyboard says so, and the log records why.
- The profile in use is sent to the keyboard again when Keystra starts and whenever the keyboard is plugged back in, so changes made while it was unplugged aren't lost.
- The profile in use can't be deleted; switch to another one first.
- A switch that doesn't reach the keyboard shows "Try again" on that profile instead of a greyed-out "In use".
- Duplicated and new profiles get a free name that always fits, like "Gaming copy 2".
- Pages still show their settings when profiles are read-only, with a note saying why.
- Tray icon: right-click to switch profiles or quit, click to open the window. Its tooltip shows the profile in use. Profiles stay locked in the tray until first-run setup is done, so a tray click can't replace the keyboard's settings before they're saved.
- When the profile in use didn't reach the keyboard, the tray and the Profiles page both offer to send it again.
- Closing the window keeps Keystra running in the tray, and the hidden window stops drawing so it uses almost no power.
- Opening Keystra while it's already running brings the running window to the front instead of starting a second copy.
- Settings page with an option to start Keystra with Windows, off by default. It starts in the tray, and only your own Windows account is changed.
- Keystra notices the keyboard being plugged in or pulled out the moment Windows does. After a USB error it looks for the keyboard again right away. If the errors keep coming, it waits longer each time and says so on every page.
- Custom per-key colors: on the Lighting page's Custom effect, click keys on the keyboard picture to select them, pick a color, and Save pattern writes one of the keyboard's 5 patterns, confirmed by reading it back. Saving a pattern the keyboard already holds writes nothing. The picture shows the keyboard's real saved patterns.
- A copy of all 5 patterns is kept on this PC, read once when the keyboard connects. If a pattern changed elsewhere, Keystra offers to put its copy back or keep the keyboard's version.
- The keyboard picture plays a looping example of each effect (sweeps, rings, key presses, twinkles, falling light), following its color, speed and direction. It pauses when minimized or scrolled out of view, and shows a still moment of the effect when Windows animations are turned off.
- When you switch effects, the direction carries over only when the new effect has the same one (Left stays Left); otherwise it starts at the first option, as its tile shows.
- Keys on the picture can be picked with the keyboard: Tab onto the picture, move with the arrow keys, and press Space or Enter.
- Starting Keystra with Windows doesn't load the window until you open it from the tray.
- A German MX 8.2 works fully; the Lighting page notes that the picture shows the US layout for now.

[Unreleased]: https://github.com/Vultify/Keystra/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Vultify/Keystra/releases/tag/v1.0.0
