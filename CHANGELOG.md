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
- Warning when MagCrate is running, since both apps share the keyboard's reply channel, plus retries that recover when the two apps' replies get mixed up.
- Lighting mode names match MagCrate's, and Music Sync's rainbow and color settings read correctly.
- The never-send list is checked again on the exact bytes at the moment they're sent.
- A keyboard that fails to answer is retried after a growing wait instead of every second.
