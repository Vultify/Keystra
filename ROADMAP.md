# Keystra Roadmap

The build order for Keystra. Each phase ends with a checkpoint: it's tested, I've checked anything that needs the real keyboard, and then it gets committed and pushed.

Protocol details live in `research/PROTOCOL-NOTES.md`.

## Phase 1: Foundation

Everything else sits on this, so it goes first.

- [x] Scaffold the Tauri app (Rust core, React + TypeScript screens, Vite)
- [x] Private GitHub repo `Keystra` with README, CHANGELOG.md, and an MIT LICENSE
- [x] Theme: dark base with the logo's purple as the accent, plus fonts and corner radius, set up in shadcn/ui before any component
- [x] App icon: square K mark cut from the logo, on a transparent background
- [x] Empty app shell: window, sidebar, and placeholder pages for Lighting, Performance, and Profiles

**Checkpoint:** the app opens with the theme and icon, and nothing talks to the keyboard yet.

## Phase 2: Talking to the keyboard (read-only)

Read before we write. Nothing in this phase changes a setting.

- [x] Crash logging first: `tauri-plugin-log` writing to the app's log folder (with rotation), a panic hook that logs before the app closes, and a React error boundary that shows a "something went wrong" message
- [x] Rust device layer: find the keyboard (VID `0x046A`, PID `0x012D`, usage page `0xFFFF`), open it, and handle unplug and replug
- [x] One owner for the keyboard: a single Rust device worker holds the HID handle and runs every command (or a whole multi-step profile apply) in order, one at a time. The UI, tray and game watcher only ever ask that worker, and it tells the UI when the device connects or disconnects.
- [x] Message builder with both checksum types, with unit tests against known packets
- [x] Command allowlist: only known commands can be sent, and the "never send" list is blocked in code with tests
- [x] Read the firmware version, current profile, polling rate, and lighting settings
- [x] Debug panel that shows the raw replies, so we can compare against MagCrate (the Diagnostics page)

**Checkpoint:** Keystra shows the keyboard's real current settings, matching what MagCrate shows.

## Phase 3: Profile test and data model

The data model decides how every later screen saves things, so it gets settled before the UI is built out.

- [x] First: generate the TypeScript types the screens use from the Rust types (ts-rs), including the status event name and level limits, so the two sides can't drift as the profile model grows
- [x] Hardware test: switch onboard profiles in MagCrate and read back what changes. Result: profiles hold key mapping and actuation; lighting and polling rate didn't change on a switch.
- [x] Profile data model based on that answer: Keystra keeps full profiles on the PC (lighting, polling rate, which onboard profile holds the keys, linked games) and applies the whole set when you switch. Any number of profiles; several can share one onboard profile.
- [x] Local storage for profiles: `profiles.json` in the app's data folder, read and written only by Rust, with a `schemaVersion` field and an upgrade step on load, safe saves (temporary file, then swap), damaged files backed up, and files from a newer Keystra left untouched. The screens use it through Tauri commands.
- [x] Leave room in the model for per-key settings: new fields use `#[serde(default)]` and bump `schemaVersion`, so version 2's per-key settings add fields instead of redoing it

**Checkpoint:** profiles save and load in the app, and the design matches how the keyboard actually behaves.

## Phase 4: First writes

The two features I use, starting with the harmless one.

- [x] Lighting: set the mode, color, rainbow, brightness, speed, and direction. Read each one back to confirm.
- [x] Polling rate: every rate the keyboard offers (8000 down to 125 Hz, all confirmed on my keyboard), read back to confirm
- [x] Profile switching on the keyboard
- [x] Remember which Keystra profile is active (saved with the profiles), and show it as "Vultify (Profile 2)" wherever the current profile appears, starting with Diagnostics' Current settings. Needed because several Keystra profiles can share one onboard profile.
- [x] Writes get their own send path: send once, wait about 100 ms, then confirm with the matching read, and resend only if the read-back differs (never the read path's automatic retries). SET commands carry their payload in the `Command` type, and stay behind the allowlist and the send-point guard.
- [x] Polling rates and lighting modes become two-way tables (code to value and back) used by both reads and writes, including the per-mode color rules

**Checkpoint (passed 2026-11-14):** I change lighting, polling rate, and profile from Keystra, and the keyboard follows. MagCrate can still reset everything.

## Phase 5: The real UI

- [x] One shared way for screens to read the keyboard's settings (a hook like `useKeyboardStatus`), moved out of the Diagnostics page
- [x] Keyboard picture that shows the current lighting
- [x] Lighting page with live preview and plain-language mode names
- [x] Performance page (polling rate, with a short note on what it does)
- [x] Profiles page: create, rename, duplicate, delete, switch
- [x] First-run screen that finds the keyboard and explains the basics in one view
- [x] Clear error states for "keyboard not found" and "keyboard is busy" (for example, when MagCrate is open)

**Checkpoint (passed 2026-11-14):** someone who has never used it can change lighting and switch profiles without help.

## Phase 6: Tray and background

- [x] Tray icon with quick profile switching
- [x] Start with Windows (optional, off by default)
- [x] Single instance: opening Keystra a second time focuses the running app instead of starting another copy (`tauri-plugin-single-instance`)
- [x] Closing the window minimizes to the tray
- [x] Notice plugging and unplugging from Windows device events instead of checking every second, since Keystra now runs all day
- [x] README status line and CHANGELOG reread

**Checkpoint (passed 2026-11-15):** profile switching works from the tray with the window closed.

## Phase 7: Custom per-key colors and effect previews

Decided 2026-11-15: per-key colors only before 1.0. The other extra settings moved to version 2.

- [x] Decode MagCrate's per-key color command (`0x0C` / `0x8C`) and the MX 8.2's key order
- [x] Read-only: show the keyboard's 5 saved patterns on the keyboard picture
- [x] Pattern editor on the Lighting page: click keys to select them (Select all, Clear), pick a color, Save pattern writes the keyboard once (Discard undoes). Never streamed.
- [x] Keep a copy of the 5 patterns on the PC so a keyboard reset can't lose them
- [x] Looping previews on the keyboard picture for each effect (sweeps, rings, demo key presses, twinkles, falling light, fades), following color, speed and direction
- [x] README status line and CHANGELOG reread

**Checkpoint:** I paint a pattern in Keystra, save it, and the keyboard shows it exactly; the picture shows each effect's kind of motion.

## Phase 8: Release

- [x] Full pre-release audit (all findings fixed)
- [x] Windows installer, built only with `npm run release` (it keeps the Windows user name out of the app)
- [x] Version 1.0.0 in `package.json` and `src-tauri/Cargo.toml` (the installer reads it from `package.json`), and a CHANGELOG in Keep a Changelog format
- [x] Make the repo public

## Version 2: Auto-switch per game

Moved after 1.0.0 (decided 2026-11-15). Profiles already store `linkedApps`, so nothing earlier has to change for it.

- [ ] Watch the active window's program
- [ ] Link programs to profiles in the Profiles page
- [ ] Switch automatically, and fall back to a default profile when the game closes
- [ ] README status line and CHANGELOG reread

**Checkpoint:** opening a linked game switches the profile within a second.

## Version 2: Extra key settings

Add these one at a time. Each one is read-only first, then gets writes.

- [ ] Actuation point per key
- [ ] Rapid trigger
- [ ] Key remapping
- [ ] Multi-action (DKS)

## Later

- 2.4 GHz dongle and Bluetooth support (wired first)
- German layout (the DE model shares the same protocol)

## Parked

- **Solid color plus ripple effect.** Not possible on stock firmware: no mode takes a base color, screen sync is one color for the whole board, and streaming per-key colors wears out flash. Revisit only if patched firmware for this keyboard appears.
