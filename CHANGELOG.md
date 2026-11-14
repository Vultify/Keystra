# Changelog

All notable changes to Keystra are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Real Lighting, Performance and Profiles pages. Lighting groups every effect with a plain description and shows a picture of the keyboard lit the same way; changes reach the keyboard as soon as a control is released. Profiles can be switched, created, renamed, duplicated and deleted.
- A welcome screen on first run that finds the keyboard and starts the first profile from its current settings.
- One clear notice on every page when the keyboard is missing, busy, or MagCrate is open.
- After a failed apply, the screen shows what the keyboard really holds. Saving a profile that reached the file but not the keyboard says so instead of reporting a failed save.
- The profile in use is sent to the keyboard again when Keystra starts and whenever the keyboard is plugged back in, so changes made while it was unplugged aren't lost.
- The profile in use can't be deleted; switch to another one first.
- A switch that doesn't reach the keyboard shows "Try again" on that profile instead of a greyed-out "In use".
- Duplicated and new profiles get a free name that always fits, like "Gaming copy 2".
- Quick changes in a row no longer show a false error, and a slider no longer jumps back while you drag it.
- Pages still show their settings when profiles are read-only, with a note saying why.
