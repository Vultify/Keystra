import { useId, type KeyboardEvent } from "react";
import { Radio } from "@base-ui/react/radio";
import { RadioGroup } from "@base-ui/react/radio-group";
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

/** Rings around a color circle: bold when chosen, faint otherwise. */
const SWATCH_RING = "ring-offset-2 ring-offset-card transition-shadow outline-none focus-visible:ring-3 focus-visible:ring-ring/50";
const swatchRing = (chosen: boolean) => (chosen ? "ring-2 ring-foreground" : "ring-1 ring-foreground/10 hover:ring-2 hover:ring-foreground/40");

/** Single-choice ToggleGroup values are arrays in Base UI. */
function single(values: string[]): string | null {
  return values[0] ?? null;
}

/** Whether dark text reads better than white on a "#RRGGBB" color. */
function isLight(hex: string): boolean {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.6;
}

/**
 * Arrow keys on a row of radios: Right and Up go forward, Left and Down go back. Handled before
 * Base UI's own keys, which wrap around at the ends and pick a value on every press.
 */
function arrowStep(event: KeyboardEvent): number | null {
  const step = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1 }[event.key];
  if (step === undefined) return null;
  event.preventDefault();
  event.stopPropagation();
  return step;
}

const radiosIn = (event: KeyboardEvent) => [...event.currentTarget.querySelectorAll<HTMLElement>('[role="radio"]')];

/** A section's heading. Its id names the control below it for screen readers. */
function SectionLabel({ id, icon: Icon, label, value }: { id: string; icon: LucideIcon; label: string; value?: string }) {
  return (
    <div className="flex items-center gap-2 text-sm font-medium">
      <Icon className="size-4 text-muted-foreground" />
      <span id={id}>{label}</span>
      {value && <span className="ml-auto text-xs font-normal text-muted-foreground tabular-nums">{value}</span>}
    </div>
  );
}

// One of the keyboard's 5 levels as a row of steps. Clicking a step sends it, and the arrow keys
// move one step at a time like a slider, stopping at the ends.
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
  const labelId = useId();
  const steps = Array.from({ length: MAX_LEVEL + 1 }, (_, step) => step);
  const onArrow = (event: KeyboardEvent) => {
    const step = arrowStep(event);
    if (step === null) return;
    const next = Math.min(MAX_LEVEL, Math.max(0, value + step));
    if (next !== value) onChange(next);
    radiosIn(event)[next]?.focus();
  };
  return (
    <div className="flex flex-col gap-2.5">
      <SectionLabel id={labelId} icon={icon} label={label} value={`${value} of ${MAX_LEVEL}`} />
      <RadioGroup<number>
        aria-labelledby={labelId}
        value={value}
        onValueChange={(next) => next !== value && onChange(next)}
        onKeyDownCapture={onArrow}
        className="grid grid-cols-5 gap-1.5"
      >
        {steps.map((step) => (
          <Radio.Root
            key={step}
            value={step}
            aria-label={`${step} of ${MAX_LEVEL}`}
            className={cn(
              "h-7 rounded-md transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
              step <= value ? "bg-primary hover:bg-primary/90" : "bg-muted hover:bg-muted-foreground/25",
            )}
            // Filled steps brighten towards the top, so the row reads as a level at a glance.
            style={step <= value ? { opacity: 0.45 + 0.55 * (step / MAX_LEVEL) } : undefined}
          />
        ))}
      </RadioGroup>
      <div className="flex justify-between text-xs text-muted-foreground" aria-hidden>
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
  const labelId = useId();
  const color = lighting.color.toUpperCase();
  const swatch = COLOR_SWATCHES.find((s) => s.color.toUpperCase() === color);
  const custom = !swatch;
  // Arrows only move between swatches; Space, Enter or a click picks one, so browsing sends nothing.
  const moveBetweenSwatches = (event: KeyboardEvent) => {
    const step = arrowStep(event);
    if (step === null) return;
    const radios = radiosIn(event);
    const at = radios.indexOf(document.activeElement as HTMLElement);
    radios[Math.min(radios.length - 1, Math.max(0, at + step))]?.focus();
  };

  return (
    <div className="flex flex-col gap-3">
      <SectionLabel
        id={labelId}
        icon={PaletteIcon}
        label="Color"
        value={lighting.rainbow ? "Rainbow" : swatch ? swatch.name : `Custom ${color}`}
      />
      <ToggleGroup
        aria-labelledby={labelId}
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
          <RadioGroup<string>
            aria-labelledby={labelId}
            value={swatch?.color ?? ""}
            onValueChange={(next) => onCommit({ color: next.toUpperCase(), rainbow: false })}
            onKeyDownCapture={moveBetweenSwatches}
            className="contents"
          >
            {COLOR_SWATCHES.map((s) => (
              <Radio.Root
                key={s.color}
                value={s.color}
                aria-label={s.name}
                title={s.name}
                className={cn("flex size-8 items-center justify-center rounded-full", SWATCH_RING, swatchRing(s === swatch))}
                style={{ backgroundColor: s.color }}
              >
                <Radio.Indicator>
                  <CheckIcon className={cn("size-4", isLight(s.color) ? "text-black" : "text-white")} strokeWidth={3} />
                </Radio.Indicator>
              </Radio.Root>
            ))}
          </RadioGroup>
          <label
            title="Pick any color"
            className={cn(
              "relative flex size-8 cursor-pointer items-center justify-center rounded-full has-focus-visible:ring-3 has-focus-visible:ring-ring/50",
              SWATCH_RING,
              swatchRing(custom),
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
  const labelId = useId();
  // Pattern numbers fit as plain digits; other options keep their names.
  const numbered = mode.optionLabel === "Pattern";
  return (
    <div className="flex flex-col gap-3">
      <SectionLabel id={labelId} icon={MoveIcon} label={mode.optionLabel ?? "Option"} value={mode.options[value]} />
      <ToggleGroup
        aria-labelledby={labelId}
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
