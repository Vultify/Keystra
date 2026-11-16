import { KeyboardIcon } from "lucide-react";

import { KeyboardNotice } from "@/components/keyboard-notice";
import { KeyboardPicture } from "@/components/keyboard-picture";
import { PatternEditorCard } from "@/components/pattern-editor";
import { ErrorAlert, NoProfileNotice } from "@/components/notices";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldGroup, FieldLabel, FieldSet, FieldLegend } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Spinner } from "@/components/ui/spinner";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useActiveProfile } from "@/hooks/use-active-profile";
import { usePatternEditor } from "@/hooks/use-pattern-editor";
import { EFFECT_GROUPS, effectGuide } from "@/lib/effect-guide";
import { isGermanBoard, MAX_LEVEL, type KeyboardStatus } from "@/lib/keyboard";
import { COLOR_SWATCHES, lightingMode, OFFERED_LIGHTING_MODES } from "@/lib/lighting";
import type { LightingSettings, Profile, ProfileList } from "@/lib/profiles";

/** How long the color picker waits after the last change before sending it. */
const COLOR_SETTLE_MS = 400;

/** Single-choice ToggleGroup values are arrays in Base UI; this keeps the pages simple. */
function single(values: string[]): string | null {
  return values[0] ?? null;
}

function LevelSlider({
  id,
  label,
  value,
  onPreview,
  onCommit,
}: {
  id: string;
  label: string;
  value: number;
  onPreview: (value: number) => void;
  onCommit: (value: number) => void;
}) {
  const pick = (next: number | readonly number[]) => (Array.isArray(next) ? next[0] : (next as number));
  return (
    <Field>
      <FieldLabel htmlFor={id}>
        {label}
        <span className="ml-auto text-muted-foreground">
          {value} of {MAX_LEVEL}
        </span>
      </FieldLabel>
      <Slider
        id={id}
        min={0}
        max={MAX_LEVEL}
        step={1}
        value={[value]}
        onValueChange={(next) => onPreview(pick(next))}
        onValueCommitted={(next) => onCommit(pick(next))}
      />
    </Field>
  );
}

// Edits the active profile's lighting: changes show on the picture at once and reach the keyboard
// when a control is released. The Custom effect also paints its saved per-key pattern.
export function LightingScreen({ status, list }: { status: KeyboardStatus | null; list: ProfileList | null }) {
  const { profile, preview, commit, commitLater, saving, problem } = useActiveProfile(list);
  const connected = status?.state === "connected";
  // The Custom effect's option is which saved pattern it shows.
  const patternSlot = profile && effectGuide(profile.lighting.mode).preview === "pattern" ? profile.lighting.direction : null;
  const editor = usePatternEditor(patternSlot, connected);
  if (!profile) return <NoProfileNotice status={status} list={list} />;
  const { lighting } = profile;
  const mode = lightingMode(lighting.mode);
  const guide = effectGuide(lighting.mode);

  const withLighting = (change: Partial<LightingSettings>): Profile => ({
    ...profile,
    lighting: { ...lighting, ...change },
  });

  function pickEffect(value: string | null) {
    if (value === null) return;
    const next = Number(value);
    const options = lightingMode(next)?.options.length ?? 0;
    commit(withLighting({ mode: next, direction: lighting.direction < options ? lighting.direction : 0 }));
  }

  // The native color picker reports every drag step, so wait for it to settle before sending.
  function pickColor(color: string) {
    commitLater(withLighting({ color: color.toUpperCase(), rainbow: false }), COLOR_SETTLE_MS);
  }

  return (
    <div className="flex flex-col gap-6">
      <KeyboardNotice status={status} />
      <ErrorAlert title="That change didn't go through" message={problem} />
      {isGermanBoard(status) && (
        <Alert>
          <KeyboardIcon />
          <AlertTitle>The picture shows the US layout</AlertTitle>
          <AlertDescription>
            Your German keyboard works fully, but the picture doesn't have its two extra keys yet, so they can't be
            painted. They keep their colors when you save a pattern.
          </AlertDescription>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            {mode?.name ?? "Unknown effect"}
            {saving && <Spinner />}
          </CardTitle>
          <CardDescription>
            {guide.description} Editing <span className="text-foreground">{profile.name}</span>.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="mx-auto w-full max-w-4xl">
            <KeyboardPicture
              lighting={lighting}
              pattern={editor.pattern}
              editing={patternSlot === null ? undefined : { selected: editor.selected, onToggle: editor.toggle }}
            />
          </div>
        </CardContent>
      </Card>

      {patternSlot !== null && <PatternEditorCard number={patternSlot + 1} editor={editor} connected={connected} />}

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <Card>
          <CardHeader>
            <CardTitle>Effect</CardTitle>
            <CardDescription>Pick how your keyboard lights up.</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            {EFFECT_GROUPS.map((group) => {
              const effects = OFFERED_LIGHTING_MODES.filter(([id]) => effectGuide(id).group === group.id);
              return (
                <FieldSet key={group.id} className="gap-1.5">
                  <FieldLegend variant="label" className="mb-0 text-muted-foreground">
                    {group.title}
                  </FieldLegend>
                  <ToggleGroup
                    variant="outline"
                    spacing={1}
                    className="flex-wrap"
                    value={[String(lighting.mode)]}
                    onValueChange={(values) => pickEffect(single(values))}
                  >
                    {effects.map(([id, effect]) => (
                      <ToggleGroupItem key={id} value={String(id)} title={effectGuide(id).description}>
                        {effect.name}
                      </ToggleGroupItem>
                    ))}
                  </ToggleGroup>
                </FieldSet>
              );
            })}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Adjust</CardTitle>
            <CardDescription>
              {mode?.brightness || mode?.speed || mode?.usesColor || mode?.optionLabel
                ? "Fine-tune the effect."
                : "This effect has nothing to adjust."}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <FieldGroup>
              {mode?.optionLabel && (
                <FieldSet>
                  <FieldLegend variant="label">{mode.optionLabel}</FieldLegend>
                  <ToggleGroup
                    variant="outline"
                    className="flex-wrap"
                    value={[String(lighting.direction)]}
                    onValueChange={(values) => {
                      const value = single(values);
                      if (value !== null) commit(withLighting({ direction: Number(value) }));
                    }}
                  >
                    {mode.options.map((option, index) => (
                      <ToggleGroupItem key={option} value={String(index)}>
                        {option}
                      </ToggleGroupItem>
                    ))}
                  </ToggleGroup>
                </FieldSet>
              )}

              {mode?.brightness && (
                <LevelSlider
                  id="lighting-brightness"
                  label="Brightness"
                  value={lighting.brightness}
                  onPreview={(brightness) => preview(withLighting({ brightness }))}
                  onCommit={(brightness) => commit(withLighting({ brightness }))}
                />
              )}
              {mode?.speed && (
                <LevelSlider
                  id="lighting-speed"
                  label="Speed"
                  value={lighting.speed}
                  onPreview={(speed) => preview(withLighting({ speed }))}
                  onCommit={(speed) => commit(withLighting({ speed }))}
                />
              )}

              {mode?.usesColor && (
                <FieldSet>
                  <FieldLegend variant="label">Color</FieldLegend>
                  <Field orientation="horizontal">
                    <Switch
                      id="lighting-rainbow"
                      checked={lighting.rainbow}
                      onCheckedChange={(rainbow) => commit(withLighting({ rainbow }))}
                    />
                    <FieldLabel htmlFor="lighting-rainbow">Rainbow</FieldLabel>
                  </Field>
                  {!lighting.rainbow && (
                    <div className="flex flex-wrap items-center gap-2">
                      <ToggleGroup
                        spacing={1}
                        className="flex-wrap"
                        value={[lighting.color]}
                        onValueChange={(values) => {
                          const color = single(values);
                          if (color) commit(withLighting({ color }));
                        }}
                      >
                        {COLOR_SWATCHES.map((swatch) => (
                          <ToggleGroupItem key={swatch.color} value={swatch.color} aria-label={swatch.name} title={swatch.name}>
                            <span className="size-4 rounded-full border" style={{ backgroundColor: swatch.color }} />
                          </ToggleGroupItem>
                        ))}
                      </ToggleGroup>
                      <Input
                        aria-label="Custom color"
                        title="Custom color"
                        type="color"
                        className="h-8 w-12 p-1"
                        value={lighting.color.toLowerCase()}
                        onChange={(event) => pickColor(event.target.value)}
                      />
                    </div>
                  )}
                </FieldSet>
              )}
            </FieldGroup>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
