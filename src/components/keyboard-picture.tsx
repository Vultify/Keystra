import { useEffect, useMemo, useRef, useState, type KeyboardEvent, type RefObject } from "react";

import { useKeyLights } from "@/hooks/use-key-lights";
import { effectGuide } from "@/lib/effect-guide";
import { effectPreview } from "@/lib/effect-motion";
import { baseColor, restFrame } from "@/lib/key-lights";
import { MAX_LEVEL } from "@/lib/keyboard";
import { BOARD_HEIGHT, BOARD_WIDTH, KEYS, keyName, neighborKey, type ArrowDirection, type Key } from "@/lib/keyboard-layout";
import { lightingMode } from "@/lib/lighting";
import type { Pattern } from "@/lib/patterns";
import type { LightingSettings } from "@/lib/profiles";

const UNIT = 44;
const GAP = 5;
const PAD = 16;

const CAP = "#141419";
const CAP_TOP = "#1d1d24";
const UNLIT = "#4a4a55";
const PRINTED = "#6c6c78";

const center = (k: Key) => ({ cx: (k.x + k.w / 2) * UNIT, cy: (k.y + 0.5) * UNIT });

/** Word legends (TAB, PG UP) are set smaller than single characters, like the real keycaps. */
const isWord = (label: string) => label.length > 1 && !/^F\d+$/.test(label);

const box = (k: Key, inset: number) => ({
  x: k.x * UNIT + GAP / 2 + inset,
  y: k.y * UNIT + GAP / 2 + inset,
  width: k.w * UNIT - GAP - inset * 2,
  height: UNIT - GAP - inset * 2,
});

function legendProps(k: Key) {
  const { cx, cy } = center(k);
  return {
    x: cx,
    y: k.sub ? cy - 2 : cy + (isWord(k.label) ? 3.5 : 5),
    fontSize: isWord(k.label) ? 9.5 : 14,
    fontWeight: 700,
    textAnchor: "middle" as const,
  };
}

function PrintedLegend({ k }: { k: Key }) {
  const { cx, cy } = center(k);
  return (
    <text x={cx} y={cy + 13} fill={PRINTED} fontSize={isWord(k.sub ?? "") ? 7.5 : 10} fontWeight={500} textAnchor="middle">
      {k.sub}
    </text>
  );
}

type Editing = {
  /** Light numbers of the selected keys. */
  selected: ReadonlySet<number>;
  onToggle: (light: number) => void;
};

const ARROWS: Record<string, ArrowDirection> = { ArrowLeft: "left", ArrowRight: "right", ArrowUp: "up", ArrowDown: "down" };

/** Whether an element is at least partly on screen, so animation can stop once it's scrolled away. */
function useOnScreen<T extends Element>(ref: RefObject<T | null>) {
  const [on, setOn] = useState(true);
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new IntersectionObserver(([entry]) => setOn(entry.isIntersecting));
    observer.observe(element);
    return () => observer.disconnect();
  }, [ref]);
  return { ref, on };
}

// A drawn MX 8.2 lit the way the lighting settings describe: light shows around each keycap's
// edge and through its top legends, like the real board, playing a looping example of the effect.
// With `editing`, keys can be clicked to select them for painting a pattern.
export function KeyboardPicture({
  lighting,
  pattern = null,
  editing,
}: {
  lighting: LightingSettings;
  pattern?: Pattern | null;
  editing?: Editing;
}) {
  const mode = lightingMode(lighting.mode);
  const off = effectGuide(lighting.mode).preview === "off";
  const level = off ? 0 : mode?.brightness === false ? 1 : lighting.brightness / MAX_LEVEL;
  const { mode: effect, direction, speed, rainbow, color } = lighting;
  const preview = useMemo(() => effectPreview({ mode: effect, direction, speed, rainbow }), [effect, direction, speed, rainbow]);
  const bases = useMemo(() => KEYS.map(baseColor({ mode: effect, rainbow, color }, pattern)), [effect, rainbow, color, pattern]);
  const restAt = useMemo(() => restFrame(preview), [preview]);
  const glows = useRef<(SVGRectElement | null)[]>([]);
  const legends = useRef<(SVGTextElement | null)[]>([]);
  const visible = useOnScreen(useRef<SVGSVGElement>(null));
  useKeyLights(preview, bases, [glows, legends], { playing: visible.on, restAt });

  // While editing, the keys are one Tab stop and the arrow keys move between them.
  const [focused, setFocused] = useState(0);
  const targets = useRef<(SVGRectElement | null)[]>([]);
  const onKey = (event: KeyboardEvent, index: number) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      editing?.onToggle(KEYS[index].light);
      return;
    }
    const direction = ARROWS[event.key];
    if (!direction) return;
    event.preventDefault();
    const next = neighborKey(index, direction);
    setFocused(next);
    targets.current[next]?.focus();
  };

  return (
    <svg
      ref={visible.ref}
      viewBox={`${-PAD} ${-PAD} ${BOARD_WIDTH * UNIT + PAD * 2} ${BOARD_HEIGHT * UNIT + PAD * 2}`}
      className="w-full select-none"
      style={{ fontFamily: "var(--font-sans)" }}
      role={editing ? "group" : "img"}
      aria-label={editing ? "Keyboard: pick keys to paint" : `Keyboard lit with ${mode?.name ?? "an unknown effect"}`}
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
        width={BOARD_WIDTH * UNIT + PAD * 2}
        height={BOARD_HEIGHT * UNIT + PAD * 2}
        rx={16}
        fill="#0b0b0e"
        stroke="var(--border)"
      />

      {/* The light that spills out around each keycap. */}
      <g filter="url(#keystra-edge-glow)" opacity={off ? 0 : 0.25 + level * 0.75}>
        {KEYS.map((k, i) => (
          <rect
            key={k.light}
            ref={(element) => {
              glows.current[i] = element;
            }}
            {...box(k, -1.5)}
            rx={7}
            fill="transparent"
          />
        ))}
      </g>

      {/* Keycaps: a dark cap with a slightly lighter top, and the printed legends the light skips. */}
      {KEYS.map((k) => (
        <g key={k.light}>
          <rect {...box(k, 0)} rx={6} fill={CAP} />
          <rect {...box(k, 3)} rx={4} fill={CAP_TOP} />
          {k.sub && <PrintedLegend k={k} />}
        </g>
      ))}

      {/* Top legends: grey when unlit, with the lit color on top. */}
      {KEYS.map((k) => (
        <text key={k.light} {...legendProps(k)} fill={UNLIT}>
          {k.label}
        </text>
      ))}
      <g filter="url(#keystra-legend-glow)" opacity={off ? 0 : 0.35 + level * 0.65}>
        {KEYS.map((k, i) => (
          <text
            key={k.light}
            ref={(element) => {
              legends.current[i] = element;
            }}
            {...legendProps(k)}
            fill="transparent"
          >
            {k.label}
          </text>
        ))}
      </g>

      {/* While editing a pattern: a click target over each key, outlined when selected. */}
      {editing &&
        KEYS.map((k, i) => {
          const selected = editing.selected.has(k.light);
          return (
            <rect
              key={k.light}
              ref={(element) => {
                targets.current[i] = element;
              }}
              {...box(k, -1)}
              rx={7}
              role="checkbox"
              aria-checked={selected}
              aria-label={keyName(k)}
              tabIndex={i === focused ? 0 : -1}
              className="cursor-pointer outline-none focus-visible:stroke-ring"
              fill="transparent"
              stroke={selected ? "var(--primary)" : "transparent"}
              strokeWidth={selected ? 3 : 2}
              onClick={() => editing.onToggle(k.light)}
              onFocus={() => setFocused(i)}
              onKeyDown={(event) => onKey(event, i)}
            />
          );
        })}
    </svg>
  );
}
