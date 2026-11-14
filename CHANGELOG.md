# Changelog

All notable changes to Keystra are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Tauri desktop app with a React and TypeScript interface.
- Dark theme built around the Keystra logo purple, with Exo 2 headings and Geist body text.
- App icon made from the Keystra K mark.
- App shell with a sidebar and placeholder pages for Lighting, Performance, and Profiles.
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
- Saved profiles: lighting, polling rate, onboard profile and linked games, stored safely on this PC with a versioned format. The Diagnostics page can save the keyboard's current settings as a profile.
- A profiles file that isn't valid JSON is kept as a backup and replaced. Any other problem with it leaves the file untouched and shows profiles as read-only. Saves are flushed to disk before they replace the old file.
- Applying a profile sends its onboard profile, polling rate and lighting to the keyboard. Keystra writes only what differs, confirms each write by reading it back, and remembers the active profile, shown as "Name (Profile N)" on Diagnostics.
- Profiles can be edited on the Diagnostics page: every lighting effect the MX 8.2 offers with its own options, plus polling rate and onboard profile.
- Quick repeated changes are merged into one write, at most one every half second, so the keyboard is never written in a stream.
- After a failed apply, the screen shows what the keyboard really holds. Saving a profile that reached the file but not the keyboard says so instead of reporting a failed save.
