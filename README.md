<p align="center">
  <img src="assets/logo-wordmark.webp" alt="Keystra" width="480">
</p>

# Keystra

A clean, simple settings app for the Cherry Xtrfy MX 8.2 Pro TMR keyboard.

I built Keystra because the official software is confusing to use, has no easy way to switch profiles, and hides what the keyboard can do. Keystra aims to be the app I wanted: every setting is easy to find and profiles switch from the tray. Later, profiles will change on their own when a game opens.

> Keystra is an independent project. It isn't made by or affiliated with Cherry.

## Status

Nearly ready for 1.0: the features planned for the first release are built, and the release itself is next. Keystra finds the keyboard over USB and has its real screens: Lighting (every built-in effect, with a picture of the keyboard playing a looping example of it, and a painter for the 5 custom per-key color patterns), Performance (polling rate) and Profiles (create, rename, duplicate, delete and switch). Profiles are saved on this PC and sent to the keyboard on every switch and whenever it's plugged back in. Keystra lives in the tray: switch profiles from its menu with the window closed, and optionally start it with Windows. See [ROADMAP.md](ROADMAP.md) for the build order.

## Planned features

- After 1.0: automatic profile switching per game
- After 1.0: per-key actuation, rapid trigger, key remapping and multi-action
- Later: the German (ISO) layout on the keyboard picture. The German MX 8.2 already works; the picture just shows the US keys, so its two extra keys can't be painted yet.

## Safety

Keystra never sends firmware updates. It only sends commands that have been decoded and tested, and it blocks everything else in code.

## Where Keystra keeps its files

- Profiles: `%APPDATA%\com.vultify.keystra\profiles.json`. If that file is ever not valid JSON, Keystra keeps it next to the new one as `profiles.damaged-<number>.json`. If it is valid JSON but can't be used, Keystra leaves it alone and shows your profiles as read-only.
- A copy of the keyboard's 5 custom color patterns: `%APPDATA%\com.vultify.keystra\patterns.json`, taken when the keyboard first connects. The patterns themselves live on the keyboard; this copy lets Keystra put one back after a keyboard reset.
- Logs: `%LOCALAPPDATA%\com.vultify.keystra\logs\keystra.log`
- Start with Windows (off unless you turn it on in Settings): a `Keystra` entry under `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`. Turning the setting off removes it.

## Building from source

You need Node.js 24, Rust (stable, MSVC), and the Visual Studio C++ Build Tools.

```bash
npm install
npm run tauri dev
```

To run the tests (this also regenerates the screens' TypeScript types in `src/lib/bindings` from the Rust types):

```bash
npm test
```

To check that the committed types still match the Rust code (fails on changed, new or leftover files):

```bash
npm run check:bindings
```

To build the Windows installer:

```bash
npm run tauri build
```

## Tech

- [Tauri 2](https://tauri.app) with a Rust core
- React, TypeScript, Tailwind CSS, and [shadcn/ui](https://ui.shadcn.com)

## License

[MIT](LICENSE)
