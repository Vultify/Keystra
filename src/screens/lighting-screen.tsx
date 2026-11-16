import { KeyboardIcon } from "lucide-react";

import { EffectPicker } from "@/components/effect-picker";
import { KeyboardNotice } from "@/components/keyboard-notice";
import { KeyboardPicture } from "@/components/keyboard-picture";
import { LightingAdjust } from "@/components/lighting-adjust";
import { PatternEditorCard } from "@/components/pattern-editor";
import { ErrorAlert, NoProfileNotice } from "@/components/notices";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { useActiveProfile } from "@/hooks/use-active-profile";
import { usePatternEditor } from "@/hooks/use-pattern-editor";
import { effectGuide } from "@/lib/effect-guide";
import { isGermanBoard, type KeyboardStatus } from "@/lib/keyboard";
import { lightingMode, OFFERED_LIGHTING_MODES } from "@/lib/lighting";
import type { LightingSettings, Profile, ProfileList } from "@/lib/profiles";

/** How long the color picker waits after the last change before sending it. */
const COLOR_SETTLE_MS = 400;

// Edits the active profile's lighting: changes show on the picture at once and reach the keyboard
// when a control is released. The Custom effect also paints its saved per-key pattern.
export function LightingScreen({ status, list }: { status: KeyboardStatus | null; list: ProfileList | null }) {
  const { profile, commit, commitLater, saving, problem } = useActiveProfile(list);
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

  function pickEffect(next: number) {
    if (next === lighting.mode) return;
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

      <div className="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_20rem]">
        <Card>
          <CardHeader>
            <CardTitle>Effect</CardTitle>
            <CardDescription>Hover a tile to see it move. Click it to use it.</CardDescription>
            <CardAction>
              <span className="text-xs text-muted-foreground tabular-nums">{OFFERED_LIGHTING_MODES.length} effects</span>
            </CardAction>
          </CardHeader>
          <CardContent>
            <EffectPicker lighting={lighting} pattern={editor.pattern} onPick={pickEffect} />
          </CardContent>
        </Card>

        <Card className="lg:sticky lg:top-0">
          <CardHeader>
            <CardTitle>Adjust</CardTitle>
            <CardDescription>
              Settings for <span className="text-foreground">{mode?.name ?? "this effect"}</span>, sent as you pick them.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <LightingAdjust
              lighting={lighting}
              mode={mode}
              onCommit={(change) => commit(withLighting(change))}
              onCustomColor={pickColor}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
