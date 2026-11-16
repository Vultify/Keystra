import type { KeyboardEvent } from "react";
import { cn } from "cn";
import {
  ArrowDownIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  ArrowUpIcon,
  CheckIcon,
  GaugeIcon,
  Maximize2Icon,
  Minimize2Icon,
  MoveIcon,
  PaletteIcon,
  PlusIcon,
  RotateCcwIcon,
  RotateCwIcon,
  SlidersHorizontalIcon,
  SunIcon,
  type LucideIcon,
} from "lucide-react";

import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty";
import { Separator } from "@/components/ui/separator";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { MAX_LEVEL } from "@/lib/keyboard";
import { COLOR_SWATCHES, type LightingMode } from "@/lib/lighting";
import type { LightingSettings } from "@/lib/profiles";

const RAINBOW = "linear-gradient(90deg, #ff3b3b, #ffb23b, #f5f53b, #3bff6e, #3bd8ff, #4b5bff, #b33bff, #ff3bb3)";
const RAINBOW_RING = "conic-gradient(#ff3b3b, #ffb23b, #f5f53b, #3bff6e, #3bd8ff, #4b5bff, #b33bff, #ff3bb3, #ff3b3b)";

/** Icons for the direction-style options; anything else shows as text only. */
const OPTION_ICONS: Record<string, LucideIcon> = {
  Right: ArrowRightIcon,
  Left: ArrowLeftIcon,
  Up: ArrowUpIcon,
  Down: ArrowDownIcon,
  Clockwise: RotateCwIcon,
  Counterclockwise: RotateCcwIcon,
  Outward: Maximize2Icon,
  Inward: Minimize2Icon,
};

/** Single-choice ToggleGroup values are arrays in Base UI. */
function single(values: string[]): string | null {
  return values[0] ?? null;
}

/** Whether dark text reads better than white on a "#RRGGBB" color. */
function isLight(hex: string): boolean {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.6;
}

function SectionLabel({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value?: string }) {
  return (
    <div className="flex items-center gap-2 text-sm font-medium">
      <Icon className="size-4 text-muted-foreground" />
      {label}
      {value && <span className="ml-auto text-xs font-normal text-muted-foreground tabular-nums">{value}</span>}
    </div>
  );
}

// One of the keyboard's 5 levels as a row of steps. Clicking a step sends it; arrow keys move one
// step at a time, like a slider.
function StepControl({
  icon,
  label,
  low,
  high,
  value,
  onChange,
}: {
  icon: LucideIcon;
  label: string;
  low: string;
  high: string;
  value: number;
  onChange: (value: number) => void;
}) {
  const steps = Array.from({ length: MAX_LEVEL + 1 }, (_, step) => step);
  const onKey = (event: KeyboardEvent) => {
    const change = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1 }[event.key];
    if (change === undefined) return;
    event.preventDefault();
    const next = Math.min(MAX_LEVEL, Math.max(0, value + change));
    if (next !== value) onChange(next);
    (event.currentTarget.querySelectorAll("button")[next] as HTMLButtonElement | undefined)?.focus();
  };

  return (
    <div className="flex flex-col gap-2.5">
      <SectionLabel icon={icon} label={label} value={`${value} of ${MAX_LEVEL}`} />
      <div role="radiogroup" aria-label={label} className="grid grid-cols-5 gap-1.5" onKeyDown={onKey}>
        {steps.map((step) => (
          <button
            key={step}
            type="button"
            role="radio"
            aria-checked={step === value}
            aria-label={`${label} ${step} of ${MAX_LEVEL}`}
            tabIndex={step === value ? 0 : -1}
            onClick={() => step !== value && onChange(step)}
            className={cn(
              "h-7 rounded-md transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
              step <= value ? "bg-primary hover:bg-primary/90" : "bg-muted hover:bg-muted-foreground/25",
            )}
            // Filled steps brighten towards the top, so the row reads as a level at a glance.
            style={step <= value ? { opacity: 0.45 + 0.55 * (step / MAX_LEVEL) } : undefined}
          />
        ))}
      </div>
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>{low}</span>
        <span>{high}</span>
      </div>
    </div>
  );
}

function ColorControl({
  lighting,
  onCommit,
  onCustom,
}: {
  lighting: LightingSettings;
  onCommit: (change: Partial<LightingSettings>) => void;
  onCustom: (color: string) => void;
}) {
  const color = lighting.color.toUpperCase();
  const swatch = COLOR_SWATCHES.find((s) => s.color.toUpperCase() === color);
  const custom = !swatch;

  return (
    <div className="flex flex-col gap-3">
      <SectionLabel
        icon={PaletteIcon}
        label="Color"
        value={lighting.rainbow ? "Rainbow" : swatch ? swatch.name : `Custom ${color}`}
      />
      <ToggleGroup
        variant="outline"
        spacing={0}
        className="w-full"
        value={[lighting.rainbow ? "rainbow" : "solid"]}
        onValueChange={(values) => {
          const choice = single(values);
          if (choice) onCommit({ rainbow: choice === "rainbow" });
        }}
      >
        <ToggleGroupItem value="solid" className="flex-1">
          Solid
        </ToggleGroupItem>
        <ToggleGroupItem value="rainbow" className="flex-1">
          Rainbow
        </ToggleGroupItem>
      </ToggleGroup>

      {lighting.rainbow ? (
        <div className="flex flex-col gap-2">
          <div className="h-2 rounded-full" style={{ background: RAINBOW }} />
          <p className="text-xs text-muted-foreground">Uses the whole spectrum instead of one color.</p>
        </div>
      ) : (
        <div className="flex flex-wrap gap-2.5">
          {COLOR_SWATCHES.map((s) => {
            const chosen = s === swatch;
            return (
              <button
                key={s.color}
                type="button"
                aria-pressed={chosen}
                aria-label={s.name}
                title={s.name}
                onClick={() => !chosen && onCommit({ color: s.color.toUpperCase(), rainbow: false })}
                className={cn(
                  "flex size-8 items-center justify-center rounded-full ring-offset-2 ring-offset-card transition-shadow outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
                  chosen ? "ring-2 ring-foreground" : "ring-1 ring-white/10 hover:ring-2 hover:ring-foreground/40",
                )}
                style={{ backgroundColor: s.color }}
              >
                {chosen && <CheckIcon className={cn("size-4", isLight(s.color) ? "text-black" : "text-white")} strokeWidth={3} />}
              </button>
            );
          })}
          <label
            title="Pick any color"
            className={cn(
              "relative flex size-8 cursor-pointer items-center justify-center rounded-full ring-offset-2 ring-offset-card transition-shadow has-focus-visible:ring-3 has-focus-visible:ring-ring/50",
              custom ? "ring-2 ring-foreground" : "ring-1 ring-white/10 hover:ring-2 hover:ring-foreground/40",
            )}
            style={{ background: custom ? color : RAINBOW_RING }}
          >
            <input
              type="color"
              aria-label="Pick any color"
              className="sr-only"
              value={color.toLowerCase()}
              onChange={(event) => onCustom(event.target.value)}
            />
            {custom ? (
              <CheckIcon className={cn("size-4", isLight(color) ? "text-black" : "text-white")} strokeWidth={3} />
            ) : (
              <span className="flex size-5 items-center justify-center rounded-full bg-card">
                <PlusIcon className="size-3.5" strokeWidth={2.5} />
              </span>
            )}
          </label>
        </div>
      )}
    </div>
  );
}

function OptionControl({ mode, value, onChange }: { mode: LightingMode; value: number; onChange: (value: number) => void }) {
  const numbered = mode.options.every((option) => /^Pattern \d$/.test(option));
  return (
    <div className="flex flex-col gap-3">
      <SectionLabel icon={MoveIcon} label={mode.optionLabel ?? "Option"} value={mode.options[value]} />
      <ToggleGroup
        variant="outline"
        spacing={0}
        className="w-full"
        value={[String(value)]}
        onValueChange={(values) => {
          const next = single(values);
          if (next !== null) onChange(Number(next));
        }}
      >
        {mode.options.map((option, index) => {
          const Icon = OPTION_ICONS[option];
          return (
            <ToggleGroupItem key={option} value={String(index)} aria-label={option} title={option} className="flex-1">
              {Icon && <Icon data-icon="inline-start" />}
              {numbered ? index + 1 : option}
            </ToggleGroupItem>
          );
        })}
      </ToggleGroup>
    </div>
  );
}

// The settings the chosen effect takes, each sent to the keyboard the moment it's picked. Only the
// controls the effect uses are shown.
export function LightingAdjust({
  lighting,
  mode,
  onCommit,
  onCustomColor,
}: {
  lighting: LightingSettings;
  mode: LightingMode | undefined;
  onCommit: (change: Partial<LightingSettings>) => void;
  onCustomColor: (color: string) => void;
}) {
  const sections = [
    mode?.brightness && (
      <StepControl
        key="brightness"
        icon={SunIcon}
        label="Brightness"
        low="Dim"
        high="Bright"
        value={lighting.brightness}
        onChange={(brightness) => onCommit({ brightness })}
      />
    ),
    mode?.speed && (
      <StepControl
        key="speed"
        icon={GaugeIcon}
        label="Speed"
        low="Slow"
        high="Fast"
        value={lighting.speed}
        onChange={(speed) => onCommit({ speed })}
      />
    ),
    mode?.usesColor && <ColorControl key="color" lighting={lighting} onCommit={onCommit} onCustom={onCustomColor} />,
    mode?.optionLabel && (
      <OptionControl key="option" mode={mode} value={lighting.direction} onChange={(direction) => onCommit({ direction })} />
    ),
  ].filter(Boolean);

  if (sections.length === 0) {
    return (
      <Empty className="border-0 p-6">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <SlidersHorizontalIcon />
          </EmptyMedia>
          <EmptyTitle>Nothing to adjust</EmptyTitle>
          <EmptyDescription>{mode?.name ?? "This effect"} has no settings. Pick another effect to light up the keys.</EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      {sections.map((section, index) => (
        <div key={index} className="flex flex-col gap-5">
          {index > 0 && <Separator />}
          {section}
        </div>
      ))}
    </div>
  );
}
