# Keyboard protocol notes

What we know about how the Cherry Xtrfy MX 8.2 Pro TMR talks to its software. Sources are MagCrate 3.0.21 (taken apart, never run) and the open RY5088 research linked at the bottom.

## Identity

| Field | Value |
|---|---|
| USB vendor ID | `0x046A` (1130, Cherry) |
| USB product ID | `0x012D` (301), wired, confirmed on the real keyboard (device ID 2494, firmware `0x0321`). `0x0141` (321) is listed next to it, likely the 2.4 GHz dongle. |
| Config interface | Usage page `0xFFFF`, usage `0x02` |
| Chip / firmware family | RongYuan **RY5088**, the same family as MonsGeek and Akko magnetic boards |
| MagCrate device names | `ry5088_cherrymx82_8k_8k` (US, id 2494), `ry5088_cherrymx82_de_8k_8k` (DE, id 3098) |
| Key layout | 87 keys, 4 layers |

MagCrate is an Electron app. The screens and message building are JavaScript (`dist/js/commonKBRY5088.*.js`). A Rust helper, `iot_driver.exe`, does the USB I/O and the live screen and music lighting.

## Message format

- 64-byte HID **feature reports**, report ID 0. Send with set-feature, read the reply with get-feature.
- Byte 0 is the command. Replies echo it back in byte 0.
- Messages are padded to at least 8 bytes. Every read (GET) uses the Bit7 checksum.
- Checksum "Bit7" (most commands): byte 7 = `255 - (sum of bytes 0..6) & 0xFF`.
- Checksum "Bit8" (lighting writes): byte 8 = `255 - (sum of bytes 0..7) & 0xFF`.
- A read is: wait about 10 ms, set-feature the request, wait about 10 ms, get-feature the reply.
- MagCrate waits about 100 ms after each write.
- The keyboard has **one reply slot** shared by every app. If MagCrate (or anything else) sends a request between Keystra's send and read, Keystra gets the other app's reply, or all zeros if the other app already took it. Keystra checks the echo byte and resends with a growing wait (25, 50, 100, 200 ms). Tested against a program reading 50 times a second: 11 of 12 full reads succeeded.

## Read replies (confirmed in MagCrate's RY5088 code)

| Cmd | Reply bytes |
|---|---|
| `0x8F` GET_USB_VERSION | 0 = `0x8F` echo, 1..4 = device ID (u32, little endian; MX 8.2 US is 2494, DE is 3098), 7..8 = firmware version (`byte8 << 8 | byte7`) |
| `0x84` GET_PROFILE | 1 = active profile index |
| `0x83` GET_REPORT | 2 = polling code: 0=8000, 1=4000, 2=2000, 3=1000, 4=500, 5=250, 6=125 Hz |
| `0x87` GET_LEDPARAM | 1 = mode, 2 = 4 - speed, 3 = brightness 0..4, 4 = high nibble direction, low nibble color choice (see below), 5..7 = RGB (`FA FF FA` means white) |

**Color choice (low nibble of byte 4) depends on the mode.** Most modes: 8 = rainbow, 7 = the stored RGB, 0..6 = a preset color. Music Sync (22) and Music Sync 3 (20): 0 = rainbow, 4 = the stored RGB. Off (0), Spectrum Cycle (3), Custom (13) and Light Shadow (21) set their own colors, so the nibble isn't a color choice. The table lives in `src-tauri/src/keyboard/lighting.rs` (`LIGHTING_MODES`).

## Commands we need (stock firmware)

| Cmd | Name | Use in Keystra |
|---|---|---|
| `0x04` / `0x84` | SET / GET_PROFILE | Profile switching. Byte 1 = profile index. |
| `0x03` / `0x83` | SET / GET_REPORT | Polling rate. The RY5088 code uses: 0=8000, 1=4000, 2=2000, 3=1000, 4=500, 5=250, 6=125 Hz. The rate code goes in byte 2 for both SET and GET. |
| `0x07` / `0x87` | SET / GET_LEDPARAM | Lighting mode. Bytes: 1 mode, 2 speed (inverted, 4 - speed), 3 brightness 0..4, 4 options (high nibble = direction, low nibble = color choice, which depends on the mode), 5..7 RGB. SET uses Bit8, GET uses Bit7. |
| `0x0C` / `0x8C` | SET / GET_USERPIC | Static per-key colors ("Custom" mode, 5 slots). 7 packets of 56 bytes. Writes flash, so it's for saving a pattern, never for animation. Decoded below. |
| `0x09` / `0x89` | SET / GET_KBOPTION | Keyboard options (one shared payload). |
| `0x0A` / `0x8A` | SET / GET_KEYMATRIX | Key remapping. |
| `0x65` / `0xE5` | SET / GET_MULTI_MAGNETISM | Per-key actuation, rapid trigger, DKS. |
| `0x8F` | GET_USB_VERSION | Device ID and firmware version. Safe first handshake. |

## Writes (from MagCrate's RY5088 code, built the same way in Keystra)

**Hardware check (2026-11-14):** switching onboard profile, polling rate (including 125 Hz) and lighting (including an effect direction) from Keystra all worked and read back correctly, so Cherry's firmware matches these bytes.

MagCrate sends a write with set-feature only (no reply read), then waits about 100 ms. Keystra does the same, then confirms with the matching GET and resends once if the keyboard didn't keep it.

| Write | Bytes | Checksum |
|---|---|---|
| `04` SET_PROFILE | `04 slot` (slot 0 to 3) | Bit7 |
| `03` SET_REPORT | `03 00 code` (same codes as GET_REPORT) | Bit7 |
| `07` SET_LEDPARAM | `07 mode (4 - speed) brightness option R G B` | Bit8 |

Lighting option byte: high nibble = the effect's option (direction, or the pattern for Custom), low nibble = color choice. MagCrate writes 8 (rainbow) or 7 (stored RGB) for most effects, 0 or 4 for Music Sync, `option << 4` with RGB `00 C8 C8` for Custom, and a 0 option byte for Light Shadow. White `FF FF FF` is sent as `FA FF FA`. One difference: for Off and Spectrum Cycle, which pick their own colors, MagCrate still writes 8 or 7 from its rainbow flag, while Keystra always writes 7.

**The MX 8.2's lighting layout** (MagCrate's `lightingLayout` for device 2494). Options are listed in firmware order, translated from MagCrate's Chinese labels:

| Effect | Brightness | Speed | Color | Options |
|---|---|---|---|---|
| Off (0) | | | | |
| Constant Light (1) | yes | | yes | |
| Spectrum Cycle (3) | yes | yes | | |
| Flowing Light (4) | yes | yes | yes | Right, Left, Down, Up |
| Flow (7) | yes | yes | yes | Zigzag, Spiral |
| Color Spring (11) | yes | yes | yes | Outward, Inward |
| Neon (12) | yes | yes | yes | Right, Left |
| Custom (13) | yes | | | Pattern 1 to 5 |
| Loop (15) | yes | yes | yes | Counterclockwise, Clockwise |
| Light Shadow (21) | yes | | | |
| Music Sync (22) | yes | | yes | Upright, Separate, Intersect |
| Every other effect (2, 5, 6, 8, 9, 10, 14, 16 to 19) | yes | yes | yes | |

Music Sync 3 (20) isn't in the MX 8.2's layout, so Keystra reads it but never writes it. Light Shadow and Music Sync take their colors from the screen or sound through MagCrate's helper, so on their own they may not show much.

## Custom per-key colors (`0x0C` / `0x8C`)

From MagCrate's `_setLightPic`, `_getLightPic` and `getLightPic` in `commonKBRY5088.*.js`, and the MX 8.2's own `defaultMatrix` in `ry5088_cherrymx82_8k_8k.*.js`. Reads and writes confirmed on my keyboard on 2026-11-15.

- **5 slots (0 to 4)**, the same numbers the Custom effect's option uses (Pattern 1 is slot 0). Slot 0 holds red W A S D and slot 1 red 1 to 6, Q W E R and A S D F from the factory.
- **Colors are 3 bytes (R, G, B) per light, in light-number order.** Light number = column x 6 + row. Rows top to bottom: function row, number row, Tab row, Caps row, Shift row, Ctrl row. Columns run left to right; see the table below.
- **Read (`0x8C`):** request bytes 0 = `0x8C`, 1 = slot, 2 = `0xFF`, 3 = page (0 to 5), Bit7 checksum. Each reply is 64 bytes of raw color data with **no echo byte**, so pages 0 to 5 joined give 384 bytes (128 lights). With no echo, a reply mixed up with another app's can't be spotted from its first byte; Keystra reads the pattern twice and only trusts matching reads.
- **Write (`0x0C`):** 7 packets. Bytes 0 = `0x0C`, 1 = slot, 2 = `0xFF`, 3 = packet (0 to 6), 4 = data length (56, or 42 for packet 6), 5 = 1 on the last packet, 7 = Bit7 checksum of bytes 0 to 6, then 56 data bytes (zero padded). 378 bytes in all, lights 0 to 125. MagCrate waits after the last packet. This writes flash, so Keystra writes only on Save pattern, never per click.

**Light numbers on the MX 8.2 (US).** Gaps (`-`) have no key. 10 and 75 are the ISO-only keys, absent on the US board.

| Column | Function row | Number row | Tab row | Caps row | Shift row | Ctrl row |
|---|---|---|---|---|---|---|
| 0 (0 to 5) | Esc | ` | Tab | Caps | LShift | LCtrl |
| 1 (6 to 11) | - | 1 | Q | A | (ISO \\) | LWin |
| 2 (12 to 17) | F1 | 2 | W | S | Z | LAlt |
| 3 (18 to 23) | F2 | 3 | E | D | X | - |
| 4 (24 to 29) | F3 | 4 | R | F | C | - |
| 5 (30 to 35) | F4 | 5 | T | G | V | - |
| 6 (36 to 41) | F5 | 6 | Y | H | B | Space |
| 7 (42 to 47) | F6 | 7 | U | J | N | - |
| 8 (48 to 53) | F7 | 8 | I | K | M | - |
| 9 (54 to 59) | F8 | 9 | O | L | , | - |
| 10 (60 to 65) | F9 | 0 | P | ; | . | RAlt |
| 11 (66 to 71) | F10 | Minus | [ | ' | / | Fn |
| 12 (72 to 77) | F11 | = | ] | (ISO #) | RShift | Menu |
| 13 (78 to 83) | F12 | Backspace | \\ | Enter | - | RCtrl |
| 14 (84 to 89) | PrtSc | Insert | Delete | - | - | Left |
| 15 (90 to 95) | ScrLk | Home | End | - | Up | Down |
| 16 (96 to 101) | Pause | PgUp | PgDn | - | - | Right |

## Profiles

- 4 onboard profiles, numbered 0 to 3 (MagCrate shows them as key layers 1 to 4). `0x04` SET_PROFILE switches, `0x84` GET_PROFILE reads the active one.
- **Per profile:** key mapping (MagCrate reads `getKeyConfig(profile, ...)` for each one) and magnetic actuation, rapid trigger and DKS (`0xE5` reads carry a profile byte).
- **Shared by all profiles:** lighting (`0x87`) and polling rate (`0x83`). MagCrate reads both without a profile number and never re-reads them after a switch. The open RY5088 research also lists LED settings in one shared config block. **Hardware check (2026-11-14):** switching profile 1 to 2 in MagCrate, Keystra read the new profile correctly while lighting (Color Grid) and polling (2000 Hz) stayed the same. Not yet tested: changing lighting on one profile only. Keystra applies lighting and polling itself on every switch, so it works either way.
- **Fn shortcuts on the MX 8.2** (from MagCrate's default Fn table for this model): **Fn+K** cycles profiles (`profile_loop`, `08 00 03 00`), Fn+F10 is the Home key (opens the browser), Fn+F11 is My Computer, Fn+1..4 look like connection-mode switches. On my keyboard Fn+K did not change the profile in the hardware check, so the firmware's real Fn layer may differ from MagCrate's default table.
- The MonsGeek boards switch profiles with Fn+F9 to F12 and send an input report `05 01 [profile]` when that happens (open RY5088 research; not seen on the MX 8.2).

## Lighting modes on this keyboard

MX 8.2 supports these modes, by ID with MagCrate's English names (Keystra keeps these names and adds a plain description of each in `src/lib/effect-guide.ts`): Off 0, Constant Light 1, Breathing 2, Spectrum Cycle 3, Flowing Light 4, Water Ripple 5, Star Dots 6, Flow 7, Shadow 8, Layers 9, Sine Wave 10, Color Spring 11, Neon 12, Custom 13, Radiant 14, Loop 15, Color Grid 16, Snowfall 17, Meteor 18, Silent Snow 19, Light Shadow 21, Music Sync 22. (Mode 20, Music Sync 3, exists in the firmware family but isn't offered on the MX 8.2.)

Every mode takes **one** color or rainbow. No mode has a separate base color.

## The solid-color plus ripple effect

| Path | Result |
|---|---|
| A. Built-in mode with a base color | **No.** No mode takes two colors. |
| B1. Screen Sync (`0x0E`) streamed by the app | **No.** It sends one color for the whole board, not per key. |
| B2. Stream per-key frames with `0x0C` | **Unsafe.** The RY5088 researchers found continuous streaming this way wears out the flash memory. |
| B3. Per-key live stream `0xE8` | **Patched firmware only.** Not available on stock firmware. |

So on stock firmware the exact effect isn't possible without risking the keyboard.

## Never send

Blocked in Keystra no matter what. Keystra only sends commands from an allowlist, and these are also rejected in code as a second guard:

| Cmd | What it does |
|---|---|
| `0x01` | Factory reset (MagCrate stays the reset tool) |
| `0x1C`, `0x1E` | Magnetism calibration |
| `0x30`, `0x31`, `0xB0`, `0xB1` | OLED bootloader |
| `0x32`, `0xB2` | Screen flash data |
| `0x50` | SET_SKU |
| `0x7F` | **Enter USB bootloader** (`7F 55 AA 55 AA`, the first step of a firmware update) |
| `0xAC` | Flash chip erase |
| `0xBA` | Firmware upgrade data (`BA C0` start, `BA C2` finish) |
| `0xF7`, `0xF8` | Wireless receiver status and **enter receiver bootloader** |

Keystra also never opens the bootloader-mode devices MagCrate lists (VID `0x046A` with PID `0x012E` or `0x0130`).

The high bit does **not** mean a command is a harmless read: `0xAC`, `0xB0`..`0xB2`, `0xBA`, `0xF7` and `0xF8` all have it. Only the explicit allowlist decides what may be sent.

## Open questions

- The 2.4 GHz dongle path is not covered yet. Wired first.
- Whether Cherry's firmware build matches the MonsGeek one byte for byte beyond the four writes confirmed so far (onboard profile, polling rate, lighting, per-key pattern). Verify each new command with a read (GET) before any write.

## Sources

- MagCrate 3.0.21 installer from cherry.de, unpacked in `research/vendor/` (not committed).
- [monsgeek-akko-linux](https://github.com/echtzeit-solutions/monsgeek-akko-linux): RY5088 protocol spec, `docs/PROTOCOL.md` and `TODO.md`. **No license**, so read it for facts only and don't copy its code.
