<p align="center">
  <img src="assets/logo-wordmark.webp" alt="Keystra" width="480">
</p>

# Keystra

A clean, simple settings app for the Cherry Xtrfy MX 8.2 Pro TMR keyboard.

I built Keystra because the official software is confusing to use, has no easy way to switch profiles, and hides what the keyboard can do. Keystra aims to be the app I wanted: every setting is easy to find, profiles switch from the tray, and profiles change on their own when a game opens.

> Keystra is an independent project. It isn't made by or affiliated with Cherry.

## Status

Early development. Keystra finds the keyboard over USB, saves profiles on this PC, and applies them: it switches the onboard profile and sets the polling rate and lighting. For now this all lives on a Diagnostics test page; the real Lighting, Performance and Profiles screens come next. See [ROADMAP.md](ROADMAP.md) for the build order.

## Planned features

- Lighting: every built-in effect, color, brightness, and speed
- Polling rate, up to 8000 Hz
- Profiles with tray switching and automatic switching per game
- Later: per-key actuation, rapid trigger, key remapping, and custom per-key colors

## Safety

Keystra never sends firmware updates. It only sends commands that have been decoded and tested, and it blocks everything else in code.

## Where Keystra keeps its files

- Profiles: `%APPDATA%\com.vultify.keystra\profiles.json`. If that file is ever not valid JSON, Keystra keeps it next to the new one as `profiles.damaged-<number>.json`. If it is valid JSON but can't be used, Keystra leaves it alone and shows your profiles as read-only.
- Logs: `%LOCALAPPDATA%\com.vultify.keystra\logs\keystra.log`

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
