<p align="center">
  <img src="assets/logo-wordmark.webp" alt="Keystra" width="480">
</p>

# Keystra

A clean, simple settings app for the Cherry Xtrfy MX 8.2 Pro TMR keyboard.

I built Keystra because the official software is confusing to use, has no easy way to switch profiles, and hides what the keyboard can do. Keystra aims to be the app I wanted: every setting is easy to find and profiles switch from the tray. Later, profiles will change on their own when a game opens.

> Keystra is an independent project. It isn't made by or affiliated with Cherry.

## Status

Version 1.0.0. Keystra finds the keyboard over USB and has its real screens: Lighting (every built-in effect, with a picture of the keyboard playing a looping example of it, and a painter for the 5 custom per-key color patterns), Performance (polling rate) and Profiles (create, rename, duplicate, delete and switch). Profiles are saved on this PC and sent to the keyboard on every switch and whenever it's plugged back in. Keystra lives in the tray: switch profiles from its menu with the window closed, and optionally start it with Windows. See [ROADMAP.md](ROADMAP.md) for the build order.

## Install

1. Download `Keystra_1.0.0_x64-setup.exe` from the [Releases page](https://github.com/Vultify/Keystra/releases).
2. Run it. Windows may show "Windows protected your PC", because the installer isn't code-signed. Click **More info**, then **Run anyway**.
3. Plug in your MX 8.2 Pro TMR with the USB cable and open Keystra. Wireless isn't supported yet.

Or install it from PowerShell. This downloads the same installer and runs it silently:

```powershell
irm https://github.com/Vultify/Keystra/releases/download/v1.0.0/Keystra_1.0.0_x64-setup.exe -OutFile "$env:TEMP\keystra-setup.exe"; & "$env:TEMP\keystra-setup.exe" /S
```

Keystra installs for your Windows account only and needs no administrator rights. It runs on Windows 10 and 11. Keystra's window uses Microsoft Edge WebView2, which comes with Windows 11; on Windows 10 the installer downloads it if it's missing, so it needs an internet connection the first time. Close MagCrate while you use Keystra, since the two share the keyboard's connection.

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
npm start
```

`npm start` (or `npm run tauri dev`) runs a development build named "Keystra Dev". It has its own settings folder (`%APPDATA%\com.vultify.keystra.dev`), its own start-with-Windows entry and its own single-instance lock, so it never touches an installed Keystra or its profiles. Only the release build is named Keystra.

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
npm run release
```

This rewrites the builder's home folder out of the file paths Rust stores inside the app, then checks the finished app and installer and stops if that folder's name is still in them.

## Tech

- [Tauri 2](https://tauri.app) with a Rust core
- React, TypeScript, Tailwind CSS, and [shadcn/ui](https://ui.shadcn.com)

## License

[MIT](LICENSE)
