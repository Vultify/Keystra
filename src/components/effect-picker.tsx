import { useMemo, useRef, useState } from "react";
import { cn } from "cn";
import { CheckIcon } from "lucide-react";

import { Separator } from "@/components/ui/separator";
import { useKeyLights } from "@/hooks/use-key-lights";
import { LIGHT_COUNT } from "@/lib/bindings/constants";
import { EFFECT_GROUPS, effectGuide } from "@/lib/effect-guide";
import { effectPreview } from "@/lib/effect-motion";
import { baseColor, restFrame } from "@/lib/key-lights";
import { BOARD_HEIGHT, BOARD_WIDTH, KEYS } from "@/lib/keyboard-layout";
import { OFFERED_LIGHTING_MODES } from "@/lib/lighting";
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

// A tiny drawing of the board playing one effect. It rests on the effect's best frame and plays
// while `playing`, using the same motion as the big picture.
function MiniKeyboard({ lighting, pattern, playing }: { lighting: LightingSettings; pattern: Pattern; playing: boolean }) {
  const { mode, direction, speed, rainbow, color } = lighting;
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

function EffectTile({
  id,
  name,
  selected,
  lighting,
  pattern,
  onPick,
}: {
  id: number;
  name: string;
  selected: boolean;
  lighting: LightingSettings;
  pattern: Pattern;
  onPick: (mode: number) => void;
}) {
  const [playing, setPlaying] = useState(false);
  const guide = effectGuide(id);
  // Each tile shows its own effect in the profile's colors and speed. Only the chosen effect keeps
  // the profile's direction, since directions mean different things per effect.
  const tileLighting = useMemo(
    () => ({ ...lighting, mode: id, direction: selected ? lighting.direction : 0 }),
    [lighting, id, selected],
  );

  return (
    <button
      type="button"
      aria-pressed={selected}
      title={guide.description}
      onClick={() => onPick(id)}
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
        <MiniKeyboard lighting={tileLighting} pattern={pattern} playing={playing} />
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
}

// Every effect as a tile with a small moving example, grouped like the guide. Hovering or focusing
// a tile plays it; clicking it switches the keyboard to that effect.
export function EffectPicker({
  lighting,
  pattern,
  onPick,
}: {
  lighting: LightingSettings;
  pattern: Pattern | null;
  onPick: (mode: number) => void;
}) {
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
              {effects.map(([id, effect]) => (
                <EffectTile
                  key={id}
                  id={id}
                  name={effect.name}
                  selected={id === lighting.mode}
                  lighting={lighting}
                  pattern={shownPattern}
                  onPick={onPick}
                />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
