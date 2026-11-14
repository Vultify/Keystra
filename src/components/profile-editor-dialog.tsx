import { useState, type FormEvent } from "react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Spinner } from "@/components/ui/spinner";
import { Switch } from "@/components/ui/switch";
import { MAX_LEVEL, onboardSlotLabel } from "@/lib/keyboard";
import { lightingMode, OFFERED_LIGHTING_MODES } from "@/lib/lighting";
import { logError } from "@/lib/logging";
import { ONBOARD_SLOTS, POLLING_RATES, type LightingSettings, type Profile } from "@/lib/profiles";

const EFFECT_ITEMS = OFFERED_LIGHTING_MODES.map(([id, mode]) => ({ value: String(id), label: mode.name }));
const POLLING_ITEMS = POLLING_RATES.map((hz) => ({ value: String(hz), label: `${hz} Hz` }));
const SLOT_ITEMS = Array.from({ length: ONBOARD_SLOTS }, (_, slot) => ({
  value: String(slot),
  label: onboardSlotLabel(slot),
}));

function SettingSelect({
  id,
  label,
  items,
  value,
  onChange,
}: {
  id: string;
  label: string;
  items: { value: string; label: string }[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      <Select items={items} value={value} onValueChange={(next) => next !== null && onChange(next)}>
        <SelectTrigger id={id} className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {items.map((item) => (
              <SelectItem key={item.value} value={item.value}>
                {item.label}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </Field>
  );
}

function LevelSlider({ id, label, value, onChange }: { id: string; label: string; value: number; onChange: (value: number) => void }) {
  return (
    <Field>
      <FieldLabel htmlFor={id}>
        {label}: {value} of {MAX_LEVEL}
      </FieldLabel>
      <Slider
        id={id}
        min={0}
        max={MAX_LEVEL}
        step={1}
        value={[value]}
        onValueChange={(next) => onChange(Array.isArray(next) ? next[0] : next)}
      />
    </Field>
  );
}

// Test bench editor for one profile's settings. The real Lighting and Performance pages replace it.
export function ProfileEditorDialog({
  profile,
  onClose,
  onSave,
}: {
  profile: Profile;
  onClose: () => void;
  onSave: (profile: Profile) => Promise<void>;
}) {
  const [draft, setDraft] = useState(profile);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { lighting } = draft;
  const mode = lightingMode(lighting.mode);

  function setLighting(change: Partial<LightingSettings>) {
    setDraft((current) => ({ ...current, lighting: { ...current.lighting, ...change } }));
  }

  function setEffect(value: string) {
    const next = Number(value);
    const options = lightingMode(next)?.options.length ?? 0;
    // Keep the option only if the new effect has it.
    setLighting({ mode: next, direction: lighting.direction < options ? lighting.direction : 0 });
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSave(draft);
      onClose();
    } catch (err) {
      setError(String(err));
      logError("saving the profile failed", err);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <form onSubmit={submit} className="flex flex-col gap-6">
          <DialogHeader>
            <DialogTitle>Edit {profile.name}</DialogTitle>
            <DialogDescription>Saving an active profile also sends it to the keyboard.</DialogDescription>
          </DialogHeader>

          <FieldGroup>
            <SettingSelect id="edit-effect" label="Lighting effect" items={EFFECT_ITEMS} value={String(lighting.mode)} onChange={setEffect} />

            {mode?.optionLabel && (
              <SettingSelect
                id="edit-option"
                label={mode.optionLabel}
                items={mode.options.map((label, index) => ({ value: String(index), label }))}
                value={String(lighting.direction)}
                onChange={(value) => setLighting({ direction: Number(value) })}
              />
            )}

            {mode?.brightness && (
              <LevelSlider id="edit-brightness" label="Brightness" value={lighting.brightness} onChange={(brightness) => setLighting({ brightness })} />
            )}
            {mode?.speed && (
              <LevelSlider id="edit-speed" label="Speed" value={lighting.speed} onChange={(speed) => setLighting({ speed })} />
            )}

            {mode?.usesColor && (
              <Field orientation="horizontal">
                <Switch id="edit-rainbow" checked={lighting.rainbow} onCheckedChange={(rainbow) => setLighting({ rainbow })} />
                <FieldLabel htmlFor="edit-rainbow">Rainbow</FieldLabel>
                {!lighting.rainbow && (
                  <Input
                    aria-label="Color"
                    type="color"
                    className="ml-auto w-16 p-1"
                    value={lighting.color.toLowerCase()}
                    onChange={(event) => setLighting({ color: event.target.value.toUpperCase() })}
                  />
                )}
              </Field>
            )}

            <SettingSelect
              id="edit-polling"
              label="Polling rate"
              items={POLLING_ITEMS}
              value={String(draft.pollingRateHz)}
              onChange={(value) => setDraft({ ...draft, pollingRateHz: Number(value) })}
            />
            <SettingSelect
              id="edit-slot"
              label="Onboard profile (key mapping and actuation)"
              items={SLOT_ITEMS}
              value={String(draft.onboardSlot)}
              onChange={(value) => setDraft({ ...draft, onboardSlot: Number(value) })}
            />
          </FieldGroup>

          {error && (
            <Alert variant="destructive">
              <AlertTitle>Couldn't save</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={saving}>
              {saving && <Spinner data-icon="inline-start" />}
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
