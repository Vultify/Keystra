/**
 * One keycap: position and width in key units, the backlit legend, the printed legend under it
 * that the light doesn't shine through (the digit on a number key, the Mac name on a modifier),
 * and its light number in the keyboard's per-key patterns (column x 6 + row).
 */
export type Key = { x: number; y: number; w: number; label: string; sub?: string; light: number };

/**
 * A run of keys on one row, with each key's light number in `lights` (same order). A number in
 * `items` sets the next key's width (default 1), and "gap" skips a unit.
 */
function row(y: number, x: number, items: (string | number)[], lights: number[]): Key[] {
  const keys: Key[] = [];
  let width = 1;
  for (const item of items) {
    if (typeof item === "number") {
      width = item;
      continue;
    }
    if (item !== "gap") {
      const [label, sub] = item.split("\n");
      keys.push({ x, y, w: width, label, sub, light: lights[keys.length] });
    }
    x += width;
    width = 1;
  }
  if (keys.length !== lights.length) throw new Error(`row at y=${y} has ${keys.length} keys but ${lights.length} lights`);
  return keys;
}

// The MX 8.2 Pro TMR's US tenkeyless layout with its own legends, matched to the real keyboard.
// Light numbers come from MagCrate's table for the MX 8.2 (research/PROTOCOL-NOTES.md).
export const KEYS: Key[] = [
  ...row(0, 0, ["ESC", "gap", "F1", "F2", "F3", "F4"], [0, 12, 18, 24, 30]),
  ...row(0, 6.5, ["F5", "F6", "F7", "F8"], [36, 42, 48, 54]),
  ...row(0, 11, ["F9\nwin lk", "F10", "F11", "F12"], [60, 66, 72, 78]),
  ...row(0, 15.25, ["PRTSC", "SCROLL", "PAUSE"], [84, 90, 96]),
  ...row(
    1.25,
    0,
    ["~\n`", "!\n1", "@\n2", "#\n3", "$\n4", "%\n5", "^\n6", "&\n7", "*\n8", "(\n9", ")\n0", "_\n-", "+\n=", 2, "←"],
    [1, 7, 13, 19, 25, 31, 37, 43, 49, 55, 61, 67, 73, 79],
  ),
  ...row(1.25, 15.25, ["INS", "HOME", "PG UP"], [85, 91, 97]),
  ...row(
    2.25,
    0,
    [1.5, "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "{\n[", "}\n]", 1.5, "|\n\\"],
    [2, 8, 14, 20, 26, 32, 38, 44, 50, 56, 62, 68, 74, 80],
  ),
  ...row(2.25, 15.25, ["DEL", "END", "PG DN"], [86, 92, 98]),
  ...row(
    3.25,
    0,
    [1.75, "CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", ":\n;", "\"\n'", 2.25, "ENTER"],
    [3, 9, 15, 21, 27, 33, 39, 45, 51, 57, 63, 69, 81],
  ),
  ...row(
    4.25,
    0,
    [2.25, "SHIFT", "Z", "X", "C", "V", "B", "N", "M", "<\n,", ">\n.", "?\n/", 2.75, "SHIFT"],
    [4, 16, 22, 28, 34, 40, 46, 52, 58, 64, 70, 76],
  ),
  ...row(4.25, 16.25, ["△"], [94]),
  ...row(
    5.25,
    0,
    [1.25, "CTRL", 1.25, "WIN\nOPT", 1.25, "ALT\nCMD", 6.25, "", 1.25, "ALT\nCMD", 1.25, "FN", 1.25, "≡\nOPT", 1.25, "CTRL"],
    [5, 11, 17, 41, 65, 71, 77, 83],
  ),
  ...row(5.25, 15.25, ["◁", "▽", "▷"], [89, 95, 101]),
];

/** Width and height of the board in key units. */
export const BOARD_WIDTH = 18.25;
export const BOARD_HEIGHT = 6.25;

/** The light number of every key, for Select all. */
export const KEY_LIGHTS: number[] = KEYS.map((key) => key.light);

/** A key's name for screen readers: its main legend, or "Space" for the blank bar. */
export function keyName(key: Key): string {
  return key.label === "" ? "Space" : key.label;
}
