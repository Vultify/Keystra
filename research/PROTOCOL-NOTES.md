# Keyboard protocol notes

What we know about how the Cherry Xtrfy MX 8.2 Pro TMR talks to its software. Sources are MagCrate 3.0.21 (taken apart, never run) and the open RY5088 research linked at the bottom.

## Identity

| Field | Value |
|---|---|
| USB vendor ID | `0x046A` (1130, Cherry) |
| USB product ID | `0x012D` (301), wired. `0x0141` (321) is listed next to it, likely the 2.4 GHz dongle. |
| Config interface | Usage page `0xFFFF`, usage `0x02` |
| Chip / firmware family | RongYuan **RY5088**, the same family as MonsGeek and Akko magnetic boards |
| MagCrate device names | `ry5088_cherrymx82_8k_8k` (US, id 2494), `ry5088_cherrymx82_de_8k_8k` (DE, id 3098) |
| Key layout | 87 keys, 4 layers |

MagCrate is an Electron app. The screens and message building are JavaScript (`dist/js/commonKBRY5088.*.js`). A Rust helper, `iot_driver.exe`, does the USB I/O and the live screen and music lighting.

## Message format

- 64-byte HID **feature reports**, report ID 0. Send with set-feature, read the reply with get-feature.
- Byte 0 is the command. Replies echo it back in byte 0.
- Checksum "Bit7" (most commands): byte 7 = `255 - (sum of bytes 0..6) & 0xFF`.
- Checksum "Bit8" (lighting commands): byte 8 = `255 - (sum of bytes 0..7) & 0xFF`.
- MagCrate waits about 100 ms after each write.

## Commands we need (stock firmware)

| Cmd | Name | Use in Keystra |
|---|---|---|
| `0x04` / `0x84` | SET / GET_PROFILE | Profile switching. Byte 1 = profile index. |
| `0x03` / `0x83` | SET / GET_REPORT | Polling rate. Codes: 0=8000, 1=4000, 2=2000, 3=1000, 4=500, 5=250 Hz (MagCrate's table says 125 for code 5; the open docs say 250). |
| `0x07` / `0x87` | SET / GET_LEDPARAM | Lighting mode. Bytes: 1 mode, 2 speed (inverted, 4 - speed), 3 brightness 0..4, 4 options (high nibble = direction, low nibble 7 = single color, 8 = rainbow), 5..7 RGB. Uses Bit8. |
| `0x0C` / `0x8C` | SET / GET_USERPIC | Static per-key colors ("Custom" mode, 5 slots). 7 packets of 56 bytes. Writes flash, so it's for saving a pattern, never for animation. |
| `0x09` / `0x89` | SET / GET_KBOPTION | Keyboard options (one shared payload). |
| `0x0A` / `0x8A` | SET / GET_KEYMATRIX | Key remapping. |
| `0x65` / `0xE5` | SET / GET_MULTI_MAGNETISM | Per-key actuation, rapid trigger, DKS. |
| `0x8F` | GET_USB_VERSION | Device ID and firmware version. Safe first handshake. |

## Lighting modes on this keyboard

MX 8.2 supports these modes (MagCrate's English name in brackets): Off 0, Constant 1, Breathing 2, Neon 3, Wave 4, Ripple 5 ("Water Ripple"), Raindrop 6 ("Star Dots"), Snake 7, Reactive 8 ("Shadow"), Converge 9 ("Layers"), Sine Wave 10, Kaleidoscope 11, Line Wave 12, Custom 13, Laser 14, Circle Wave 15 ("Loop"), Color Grid 16, Snowfall 17, Meteor 18, Reactive Off 19 ("Silent Snow"), Screen Sync 21 ("Light Shadow"), Music Sync 22.

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

Blocked in Keystra no matter what: `0x01` factory reset (MagCrate stays the reset tool), `0x50` SET_SKU, `0xAC` flash chip erase, `0x30`/`0x31` OLED bootloader, `0x1C`/`0x1E` magnetism calibration, and anything in the firmware upgrade or DFU paths. Keystra only sends commands from an allowlist.

## Open questions

- Confirm the wired PID (`0x012D`) in Device Manager.
- The 2.4 GHz dongle path is not covered yet. Wired first.
- Whether Cherry's firmware build matches the MonsGeek one byte for byte. Verify each command with a read (GET) before any write.

## Sources

- MagCrate 3.0.21 installer from cherry.de, unpacked in `research/vendor/` (not committed).
- [monsgeek-akko-linux](https://github.com/echtzeit-solutions/monsgeek-akko-linux): RY5088 protocol spec, `docs/PROTOCOL.md` and `TODO.md`. **No license**, so read it for facts only and don't copy its code.
