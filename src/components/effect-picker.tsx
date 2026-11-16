import { memo, useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { cn } from "cn";
import { CheckIcon } from "lucide-react";

import { Separator } from "@/components/ui/separator";
import { useKeyLights } from "@/hooks/use-key-lights";
import { LIGHT_COUNT } from "@/lib/bindings/constants";
import { EFFECT_GROUPS, effectGuide } from "@/lib/effect-guide";
import { effectPreview } from "@/lib/effect-motion";
import { baseColor, restFrame } from "@/lib/key-lights";
import { BOARD_HEIGHT, BOARD_WIDTH, KEYS } from "@/lib/keyboard-layout";
import { OFFERED_LIGHTING_MODES, optionAfterSwitch } from "@/lib/lighting";
import type { Pattern } from "@/lib/patterns";
import type { LightingSettings } from "@/lib/profiles";

const UNIT = 10;
const GAP = 1.5;

/** Shown on the Custom tile until the keyboard's own pattern is read: a soft sweep of hues. */
const DEMO_PATTERN: Pattern = (() => {
  const colors = Array<string>(LIGHT_COUNT).fill("#000000");
  for (const key of KEYS) colors[key.light] = `hsl(${Math.round(250 + (key.x / BOARD_WIDTH) * 110)} 85% 62%)`;
  return colors;
})();

type TileLook = Pick<LightingSettings, "mode" | "direction" | "speed" | "rainbow" | "color"> & {
  /** The pattern to show, for the Custom effect only. */
  pattern: Pattern | null;
};

// A tiny drawing of the board playing one effect. It rests on the effect's best frame and plays
// while `playing`, using the same motion as the big picture.
function MiniKeyboard({ mode, direction, speed, rainbow, color, pattern, playing }: TileLook & { playing: boolean }) {
  const preview = useMemo(() => effectPreview({ mode, direction, speed, rainbow }), [mode, direction, speed, rainbow]);
  const restAt = useMemo(() => restFrame(preview), [preview]);
  const bases = useMemo(() => KEYS.map(baseColor({ mode, rainbow, color }, pattern)), [mode, rainbow, color, pattern]);
  const lights = useRef<(SVGRectElement | null)[]>([]);
  useKeyLights(preview, bases, [lights], { playing, restAt });

  return (
    <svg viewBox={`0 0 ${BOARD_WIDTH * UNIT} ${BOARD_HEIGHT * UNIT}`} className="block w-full" aria-hidden>
      {KEYS.map((key, i) => {
        const box = { x: key.x * UNIT + GAP / 2, y: key.y * UNIT + GAP / 2, width: key.w * UNIT - GAP, height: UNIT - GAP };
        return (
          <g key={key.light}>
            <rect {...box} rx={1.6} fill="#1a1a21" />
            <rect
              {...box}
              rx={1.6}
              fill="transparent"
              ref={(element) => {
                lights.current[i] = element;
              }}
            />
          </g>
        );
      })}
    </svg>
  );
}

// One effect's tile. Memoized with plain-value props, so a change elsewhere on the page doesn't
// redraw all 22 tiles.
type TileProps = { name: string; selected: boolean; look: TileLook; onPick: (mode: number) => void };

const EffectTile = memo(function EffectTile({ name, selected, look, onPick }: TileProps) {
  const [playing, setPlaying] = useState(false);
  const guide = effectGuide(look.mode);
  return (
    <button
      type="button"
      aria-pressed={selected}
      title={guide.description}
      onClick={() => onPick(look.mode)}
      onPointerEnter={() => setPlaying(true)}
      onPointerLeave={() => setPlaying(false)}
      onFocus={() => setPlaying(true)}
      onBlur={() => setPlaying(false)}
      className={cn(
        "flex flex-col gap-2.5 rounded-xl border p-2 text-left transition-[border-color,background-color,box-shadow] duration-200 outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
        selected
          ? "border-primary bg-primary/10 shadow-[0_10px_30px_-12px_var(--primary)]"
          : "border-border bg-card hover:border-foreground/20 hover:bg-muted/40",
      )}
    >
      <div className="rounded-lg bg-black/70 p-1.5 ring-1 ring-white/5">
        <MiniKeyboard {...look} playing={playing} />
      </div>
      <div className="flex items-start gap-2 px-1 pb-0.5">
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-medium">{name}</div>
          <div className="truncate text-xs text-muted-foreground">{guide.hint}</div>
        </div>
        {selected && (
          <span className="mt-0.5 flex size-4.5 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground">
            <CheckIcon className="size-3" strokeWidth={3} />
          </span>
        )}
      </div>
    </button>
  );
}, sameTile);

function sameTile(a: TileProps, b: TileProps): boolean {
  const keys = ["mode", "direction", "speed", "rainbow", "color", "pattern"] as const;
  return a.name === b.name && a.selected === b.selected && a.onPick === b.onPick && keys.every((k) => a.look[k] === b.look[k]);
}

// Every effect as a tile with a small moving example, grouped like the guide. Hovering or focusing
// a tile plays it; clicking it switches the keyboard to that effect.
export function EffectPicker({
  lighting,
  pattern,
  lastUsed,
  onPick,
}: {
  lighting: LightingSettings;
  pattern: Pattern | null;
  /** The option last used on each effect, so tiles show what switching back would send. */
  lastUsed: Readonly<Record<number, number>>;
  onPick: (mode: number) => void;
}) {
  // The tiles get one callback for the page's life, which always calls the newest `onPick`.
  const latestPick = useRef(onPick);
  useLayoutEffect(() => {
    latestPick.current = onPick;
  });
  const pick = useCallback((mode: number) => latestPick.current(mode), []);
  const shownPattern = pattern ?? DEMO_PATTERN;

  return (
    <div className="flex flex-col gap-6">
      {EFFECT_GROUPS.map((group) => {
        const effects = OFFERED_LIGHTING_MODES.filter(([id]) => effectGuide(id).group === group.id);
        const headingId = `effect-group-${group.id}`;
        return (
          <section key={group.id} aria-labelledby={headingId} className="flex flex-col gap-3">
            <div className="flex items-center gap-3">
              <h3 id={headingId} className="text-xs font-semibold tracking-widest text-muted-foreground uppercase">
                {group.title}
              </h3>
              <span className="text-xs text-muted-foreground/60 tabular-nums">{effects.length}</span>
              <Separator className="flex-1" />
            </div>
            <div className="grid grid-cols-[repeat(auto-fill,minmax(8.75rem,1fr))] gap-3">
              {effects.map(([id, effect]) => {
                const selected = id === lighting.mode;
                // Each tile previews exactly what clicking it would send.
                const look: TileLook = {
                  mode: id,
                  direction: selected ? lighting.direction : optionAfterSwitch(lighting.mode, lighting.direction, id, lastUsed),
                  speed: lighting.speed,
                  rainbow: lighting.rainbow,
                  color: lighting.color,
                  pattern: effectGuide(id).preview === "pattern" ? shownPattern : null,
                };
                return <EffectTile key={id} name={effect.name} selected={selected} look={look} onPick={pick} />;
              })}
            </div>
          </section>
        );
      })}
    </div>
  );
}
