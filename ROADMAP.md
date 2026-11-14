# Keystra Roadmap

The build order for Keystra. Each phase ends with a checkpoint: it's tested, I've checked anything that needs the real keyboard, and then it gets committed and pushed.

Protocol details live in `research/PROTOCOL-NOTES.md`.

## Phase 1: Foundation

Everything else sits on this, so it goes first.

- [x] Scaffold the Tauri app (Rust core, React + TypeScript screens, Vite)
- [ ] Private GitHub repo `Keystra` with README, CHANGELOG.md, and an MIT LICENSE (files done, repo not created yet)
- [x] Theme: dark base with the logo's purple as the accent, plus fonts and corner radius, set up in shadcn/ui before any component
- [x] App icon: square K mark cut from the logo, on a transparent background
- [x] Empty app shell: window, sidebar, and placeholder pages for Lighting, Performance, and Profiles

**Checkpoint:** the app opens with the theme and icon, and nothing talks to the keyboard yet.

## Phase 2: Talking to the keyboard (read-only)

Read before we write. Nothing in this phase changes a setting.

- [ ] Crash logging first: `tauri-plugin-log` writing to the app's log folder (with rotation), a panic hook that logs before the app closes, and a React error boundary that shows a "something went wrong" message
- [ ] Rust device layer: find the keyboard (VID `0x046A`, PID `0x012D`, usage page `0xFFFF`), open it, and handle unplug and replug
- [ ] One owner for the keyboard: a single Rust device worker holds the HID handle and runs every command (or a whole multi-step profile apply) in order, one at a time. The UI, tray and game watcher only ever ask that worker, and it tells the UI when the device connects or disconnects.
- [ ] Message builder with both checksum types, with unit tests against known packets
- [ ] Command allowlist: only known commands can be sent, and the "never send" list is blocked in code with tests
- [ ] Read the firmware version, current profile, polling rate, and lighting settings
- [ ] Debug panel that shows the raw replies, so we can compare against MagCrate

**Checkpoint:** Keystra shows the keyboard's real current settings, matching what MagCrate shows.

## Phase 3: Profile test and data model

The data model decides how every later screen saves things, so it gets settled before the UI is built out.

- [ ] Hardware test: switch onboard profiles in MagCrate and read back what changes. MagCrate treats the 4 profiles as key layers, so we need to know whether lighting and polling rate are per profile or global.
- [ ] Profile data model based on that answer. If onboard profiles only hold key layers, Keystra keeps full profiles on the PC and applies the whole set when you switch.
- [ ] Local storage for profiles (name, settings, linked games): one JSON file in the app's data folder, read and written only by Rust, with a `schemaVersion` field and a small upgrade step on load. The screens use it through Tauri commands, so the tray and game watcher can read it with the window closed.
- [ ] Leave room in the model for per-key settings (colors, actuation, remaps), so Phase 8 adds fields instead of redoing it

**Checkpoint:** profiles save and load in the app, and the design matches how the keyboard actually behaves.

## Phase 4: First writes

The two features I use, starting with the harmless one.

- [ ] Lighting: set the mode, color, rainbow, brightness, speed, and direction. Read each one back to confirm.
- [ ] Polling rate: 8000 down to 250 Hz, read back to confirm
- [ ] Profile switching on the keyboard

**Checkpoint:** I change lighting, polling rate, and profile from Keystra, and the keyboard follows. MagCrate can still reset everything.

## Phase 5: The real UI

- [ ] Keyboard picture that shows the current lighting
- [ ] Lighting page with live preview and plain-language mode names
- [ ] Performance page (polling rate, with a short note on what it does)
- [ ] Profiles page: create, rename, duplicate, delete, switch
- [ ] First-run screen that finds the keyboard and explains the basics in one view
- [ ] Clear error states for "keyboard not found" and "keyboard is busy" (for example, when MagCrate is open)

**Checkpoint:** someone who has never used it can change lighting and switch profiles without help.

## Phase 6: Tray and background

- [ ] Tray icon with quick profile switching
- [ ] Start with Windows (optional, off by default)
- [ ] Single instance: opening Keystra a second time focuses the running app instead of starting another copy (`tauri-plugin-single-instance`)
- [ ] Closing the window minimizes to the tray

**Checkpoint:** profile switching works from the tray with the window closed.

## Phase 7: Auto-switch per game

- [ ] Watch the active window's program
- [ ] Link programs to profiles in the Profiles page
- [ ] Switch automatically, and fall back to a default profile when the game closes

**Checkpoint:** opening a linked game switches the profile within a second.

## Phase 8: Extra settings (lower priority)

Add these one at a time. Each one is read-only first, then gets writes.

- [ ] Custom per-key colors (the 5 saved patterns). One-time saves only, never streamed.
- [ ] Actuation point per key
- [ ] Rapid trigger
- [ ] Key remapping
- [ ] Multi-action (DKS)

## Phase 9: Release

- [ ] Run `/project-audit deep`
- [ ] Windows installer
- [ ] Version 1.0.0 in `package.json` and `src-tauri/Cargo.toml` (the installer reads it from `package.json`), and a CHANGELOG in Keep a Changelog format
- [ ] Make the repo public

## Later

- 2.4 GHz dongle and Bluetooth support (wired first)
- German layout (the DE model shares the same protocol)

## Parked

- **Solid color plus ripple effect.** Not possible on stock firmware: no mode takes a base color, screen sync is one color for the whole board, and streaming per-key colors wears out flash. Revisit only if patched firmware for this keyboard appears.
