import type { CSSProperties } from "react";

import { effectGuide } from "@/lib/effect-guide";
import { MAX_LEVEL } from "@/lib/keyboard";
import { lightingMode } from "@/lib/lighting";
import type { LightingSettings } from "@/lib/profiles";

/**
 * One keycap: position and width in key units, the backlit legend, and the printed legend under it
 * that the light doesn't shine through (the digit on a number key, the Mac name on a modifier).
 */
type Key = { x: number; y: number; w: number; label: string; sub?: string };

/** A run of keys on one row. A number sets the next key's width (default 1); "gap" skips a unit. */
function row(y: number, x: number, items: (string | number)[]): Key[] {
  const keys: Key[] = [];
  let width = 1;
  for (const item of items) {
    if (typeof item === "number") {
      width = item;
      continue;
    }
    if (item !== "gap") {
      const [label, sub] = item.split("\n");
      keys.push({ x, y, w: width, label, sub });
    }
    x += width;
    width = 1;
  }
  return keys;
}

// The MX 8.2 Pro TMR's US tenkeyless layout with its own legends, matched to the real keyboard.
const KEYS: Key[] = [
  ...row(0, 0, ["ESC", "gap", "F1", "F2", "F3", "F4"]),
  ...row(0, 6.5, ["F5", "F6", "F7", "F8"]),
  ...row(0, 11, ["F9\nwin lk", "F10", "F11", "F12"]),
  ...row(0, 15.25, ["PRTSC", "SCROLL", "PAUSE"]),
  ...row(1.25, 0, ["~\n`", "!\n1", "@\n2", "#\n3", "$\n4", "%\n5", "^\n6", "&\n7", "*\n8", "(\n9", ")\n0", "_\n-", "+\n=", 2, "←"]),
  ...row(1.25, 15.25, ["INS", "HOME", "PG UP"]),
  ...row(2.25, 0, [1.5, "TAB", "Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P", "{\n[", "}\n]", 1.5, "|\n\\"]),
  ...row(2.25, 15.25, ["DEL", "END", "PG DN"]),
  ...row(3.25, 0, [1.75, "CAPS", "A", "S", "D", "F", "G", "H", "J", "K", "L", ":\n;", "\"\n'", 2.25, "ENTER"]),
  ...row(4.25, 0, [2.25, "SHIFT", "Z", "X", "C", "V", "B", "N", "M", "<\n,", ">\n.", "?\n/", 2.75, "SHIFT"]),
  ...row(4.25, 16.25, ["△"]),
  ...row(5.25, 0, [1.25, "CTRL", 1.25, "WIN\nOPT", 1.25, "ALT\nCMD", 6.25, "", 1.25, "ALT\nCMD", 1.25, "FN", 1.25, "≡\nOPT", 1.25, "CTRL"]),
  ...row(5.25, 15.25, ["◁", "▽", "▷"]),
];

const WIDTH = 18.25;
const HEIGHT = 6.25;
const UNIT = 44;
const GAP = 5;
const PAD = 16;

const CAP = "#141419";
const CAP_TOP = "#1d1d24";
const UNLIT = "#4a4a55";
const PRINTED = "#6c6c78";

/** The color of the light under one key, and how the light moves for this effect. */
function paint(lighting: LightingSettings): { light: (key: Key, index: number) => string; motion: string | null } {
  const hueAt = (key: Key) => Math.round(((key.x + key.w / 2) / WIDTH) * 330);
  const rainbow = (key: Key) => `hsl(${hueAt(key)} 90% 60%)`;
  switch (effectGuide(lighting.mode).preview) {
    case "off":
      return { light: () => UNLIT, motion: null };
    case "pattern":
      return { light: (_, i) => `hsl(${(i * 47) % 360} 85% 62%)`, motion: null };
    case "cycle":
      return { light: () => "hsl(0 90% 60%)", motion: "keystra-hue" };
    case "outside":
      return { light: (key) => `hsl(${hueAt(key)} 45% 60%)`, motion: null };
    case "still":
      return { light: lighting.rainbow ? rainbow : () => lighting.color, motion: null };
    case "breathe":
      return lighting.rainbow ? { light: rainbow, motion: "keystra-hue" } : { light: () => lighting.color, motion: "keystra-breathe" };
    case "moving":
      return lighting.rainbow ? { light: rainbow, motion: "keystra-hue" } : { light: () => lighting.color, motion: null };
  }
}

const center = (k: Key) => ({ cx: (k.x + k.w / 2) * UNIT, cy: (k.y + 0.5) * UNIT });

/** Word legends (TAB, PG UP) are set smaller than single characters, like the real keycaps. */
const isWord = (label: string) => label.length > 1 && !/^F\d+$/.test(label);

function LitLegend({ k, fill }: { k: Key; fill: string }) {
  const { cx, cy } = center(k);
  const y = k.sub ? cy - 2 : cy + (isWord(k.label) ? 3.5 : 5);
  return (
    <text x={cx} y={y} fill={fill} fontSize={isWord(k.label) ? 9.5 : 14} fontWeight={700} textAnchor="middle">
      {k.label}
    </text>
  );
}

function PrintedLegend({ k }: { k: Key }) {
  const { cx, cy } = center(k);
  return (
    <text x={cx} y={cy + 13} fill={PRINTED} fontSize={isWord(k.sub ?? "") ? 7.5 : 10} fontWeight={500} textAnchor="middle">
      {k.sub}
    </text>
  );
}

// A drawn MX 8.2 lit the way the lighting settings describe: light shows around each keycap's
// edge and through its top legends, like the real board. It guides color and brightness, not each animation.
export function KeyboardPicture({ lighting }: { lighting: LightingSettings }) {
  const mode = lightingMode(lighting.mode);
  const off = effectGuide(lighting.mode).preview === "off";
  const level = off ? 0 : mode?.brightness === false ? 1 : lighting.brightness / MAX_LEVEL;
  const { light, motion } = paint(lighting);
  const seconds = 14 - (lighting.speed / MAX_LEVEL) * 10;
  const motionStyle: CSSProperties | undefined = motion ? { animation: `${motion} ${seconds}s linear infinite` } : undefined;
  const box = (k: Key, inset: number) => ({
    x: k.x * UNIT + GAP / 2 + inset,
    y: k.y * UNIT + GAP / 2 + inset,
    width: k.w * UNIT - GAP - inset * 2,
    height: UNIT - GAP - inset * 2,
  });

  return (
    <svg
      viewBox={`${-PAD} ${-PAD} ${WIDTH * UNIT + PAD * 2} ${HEIGHT * UNIT + PAD * 2}`}
      className="w-full"
      style={{ fontFamily: "var(--font-sans)" }}
      role="img"
      aria-label={`Keyboard lit with ${mode?.name ?? "an unknown effect"}`}
    >
      <defs>
        <filter id="keystra-edge-glow" x="-20%" y="-30%" width="140%" height="160%">
          <feGaussianBlur stdDeviation="3.5" />
        </filter>
        <filter id="keystra-legend-glow" x="-30%" y="-50%" width="160%" height="200%">
          <feGaussianBlur stdDeviation="1.6" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <rect
        x={-PAD}
        y={-PAD}
        width={WIDTH * UNIT + PAD * 2}
        height={HEIGHT * UNIT + PAD * 2}
        rx={16}
        fill="#0b0b0e"
        stroke="var(--border)"
      />

      {/* The light that spills out around each keycap. */}
      {!off && (
        <g className="keystra-motion" style={motionStyle}>
          <g filter="url(#keystra-edge-glow)" opacity={0.25 + level * 0.75}>
            {KEYS.map((k, i) => (
              <rect key={i} {...box(k, -1.5)} rx={7} fill={light(k, i)} />
            ))}
          </g>
        </g>
      )}

      {/* Keycaps: a dark cap with a slightly lighter top, and the printed legends the light skips. */}
      {KEYS.map((k, i) => (
        <g key={i}>
          <rect {...box(k, 0)} rx={6} fill={CAP} />
          <rect {...box(k, 3)} rx={4} fill={CAP_TOP} />
          {k.sub && <PrintedLegend k={k} />}
        </g>
      ))}

      {/* Top legends, lit from below. */}
      <g className="keystra-motion" style={motionStyle}>
        <g filter={off ? undefined : "url(#keystra-legend-glow)"} opacity={off ? 1 : 0.35 + level * 0.65}>
          {KEYS.map((k, i) => (
            <LitLegend key={i} k={k} fill={off ? UNLIT : light(k, i)} />
          ))}
        </g>
      </g>
    </svg>
  );
}
