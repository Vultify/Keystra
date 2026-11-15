import { useState, type FormEvent, type ReactElement } from "react";
import { CheckIcon, CopyIcon, PencilIcon, PlusIcon, RotateCcwIcon, Trash2Icon } from "lucide-react";

import { KeyboardNotice } from "@/components/keyboard-notice";
import { ErrorAlert, ReadOnlyNotice } from "@/components/notices";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useAction } from "@/hooks/use-action";
import { onboardSlotLabel, type KeyboardStatus } from "@/lib/keyboard";
import { describeLighting } from "@/lib/lighting";
import {
  applyProfile,
  createProfile,
  createProfileFromKeyboard,
  deleteProfile,
  duplicateProfile,
  MAX_NAME_LEN,
  ONBOARD_SLOTS,
  renameProfile,
  saveProfile,
  type Profile,
  type ProfileList,
} from "@/lib/profiles";

const SLOT_ITEMS = Array.from({ length: ONBOARD_SLOTS }, (_, slot) => ({
  value: String(slot),
  label: onboardSlotLabel(slot),
}));

function summary({ lighting, pollingRateHz, onboardSlot }: Profile): string {
  const { effect, option, color } = describeLighting(lighting.mode, lighting.rainbow, lighting.color, lighting.direction);
  return [[effect, option?.value, color].filter(Boolean).join(", "), `${pollingRateHz} Hz`, onboardSlotLabel(onboardSlot)].join(" · ");
}

function IconAction({ label, onClick, disabled, children }: { label: string; onClick: () => void; disabled?: boolean; children: ReactElement }) {
  return (
    <Tooltip>
      <TooltipTrigger render={<Button variant="ghost" size="icon-sm" aria-label={label} disabled={disabled} onClick={onClick} />}>
        {children}
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

type Editing = { kind: "new" } | { kind: "edit"; profile: Profile };

// Name and onboard profile for a new or existing profile. A new one starts from the keyboard's
// current settings when it's connected.
function ProfileDialog({
  editing,
  connected,
  onClose,
}: {
  editing: Editing;
  connected: boolean;
  onClose: () => void;
}) {
  const existing = editing.kind === "edit" ? editing.profile : null;
  const [name, setName] = useState(existing?.name ?? "");
  const [slot, setSlot] = useState(String(existing?.onboardSlot ?? 0));
  const { busy, error, run } = useAction("saving a profile");

  async function submit(event: FormEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    const saved = await run("save", async () => {
      if (!existing) {
        await (connected ? createProfileFromKeyboard(trimmed) : createProfile(trimmed));
        return;
      }
      const renamed = trimmed === existing.name ? existing : await renameProfile(existing.id, trimmed);
      if (Number(slot) !== existing.onboardSlot) {
        const { applyError } = await saveProfile({ ...renamed, onboardSlot: Number(slot) });
        if (applyError) throw new Error(`Saved, but the keyboard didn't take it: ${applyError}`);
      }
    });
    if (saved) onClose();
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <form onSubmit={submit} className="flex flex-col gap-6">
          <DialogHeader>
            <DialogTitle>{existing ? `Edit ${existing.name}` : "New profile"}</DialogTitle>
            <DialogDescription>
              {existing
                ? "Rename it, or pick which onboard profile holds its key mapping."
                : connected
                  ? "It starts with your keyboard's current lighting and polling rate."
                  : "It starts with the keyboard's factory settings."}
            </DialogDescription>
          </DialogHeader>
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="profile-name">Name</FieldLabel>
              <Input id="profile-name" value={name} maxLength={MAX_NAME_LEN} autoFocus onChange={(event) => setName(event.target.value)} />
            </Field>
            {existing && (
              <Field>
                <FieldLabel htmlFor="profile-slot">Onboard profile</FieldLabel>
                <Select items={SLOT_ITEMS} value={slot} onValueChange={(next) => next !== null && setSlot(next)}>
                  <SelectTrigger id="profile-slot" className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {SLOT_ITEMS.map((item) => (
                        <SelectItem key={item.value} value={item.value}>
                          {item.label}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <FieldDescription>
                  The keyboard keeps 4 sets of key mapping and actuation. Several profiles can share one.
                </FieldDescription>
              </Field>
            )}
          </FieldGroup>
          <ErrorAlert title="Couldn't save" message={error} />
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={name.trim() === "" || busy !== null}>
              {busy && <Spinner data-icon="inline-start" />}
              {existing ? "Save" : "Create"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

// Every saved profile: switch to one, or create, edit, duplicate and delete them.
export function ProfilesScreen({ status, list }: { status: KeyboardStatus | null; list: ProfileList | null }) {
  const { busy, error, run } = useAction("profile action");
  const [editing, setEditing] = useState<Editing | null>(null);
  const [deleting, setDeleting] = useState<Profile | null>(null);
  const connected = status?.state === "connected";
  const readOnly = list?.readOnly ?? null;
  const locked = busy !== null || readOnly !== null;

  if (!list) return null;

  return (
    <div className="flex flex-col gap-6">
      <KeyboardNotice status={status} />
      <ReadOnlyNotice list={list} />
      <ErrorAlert title="That didn't work" message={error} />

      <Card>
        <CardHeader>
          <CardTitle>Your profiles</CardTitle>
          <CardDescription>Each one remembers its lighting, polling rate and onboard profile.</CardDescription>
          <CardAction>
            <Button disabled={locked} onClick={() => setEditing({ kind: "new" })}>
              <PlusIcon data-icon="inline-start" />
              New profile
            </Button>
          </CardAction>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          {list.profiles.map((profile) => {
            const active = profile.id === list.activeId;
            // A profile in use that didn't reach the keyboard, from here or the tray, can be sent again.
            const retry = active && list.unsentId === profile.id;
            return (
              <div
                key={profile.id}
                className="flex items-center gap-4 rounded-lg border bg-background/40 px-4 py-3 data-[active=true]:border-primary/60 data-[active=true]:bg-primary/10"
                data-active={active}
              >
                <div className="flex min-w-0 flex-1 flex-col gap-1">
                  <div className="flex items-center gap-2 font-medium">
                    <span className="truncate">{profile.name}</span>
                    {active && <Badge>Active</Badge>}
                  </div>
                  <p className="truncate text-sm text-muted-foreground">{summary(profile)}</p>
                </div>
                <Button
                  variant={active && !retry ? "outline" : "default"}
                  size="sm"
                  disabled={(active && !retry) || !connected || locked}
                  onClick={() => run(`switch-${profile.id}`, () => applyProfile(profile.id))}
                >
                  {busy === `switch-${profile.id}` ? (
                    <Spinner data-icon="inline-start" />
                  ) : retry ? (
                    <RotateCcwIcon data-icon="inline-start" />
                  ) : (
                    active && <CheckIcon data-icon="inline-start" />
                  )}
                  {retry ? "Try again" : active ? "In use" : "Switch"}
                </Button>
                <div className="flex gap-1">
                  <IconAction label={`Edit ${profile.name}`} disabled={locked} onClick={() => setEditing({ kind: "edit", profile })}>
                    <PencilIcon />
                  </IconAction>
                  <IconAction
                    label={`Duplicate ${profile.name}`}
                    disabled={locked}
                    onClick={() => run(`copy-${profile.id}`, () => duplicateProfile(profile.id))}
                  >
                    <CopyIcon />
                  </IconAction>
                  <IconAction
                    label={active ? "Switch to another profile to delete this one" : `Delete ${profile.name}`}
                    disabled={locked || active || list.profiles.length === 1}
                    onClick={() => setDeleting(profile)}
                  >
                    <Trash2Icon />
                  </IconAction>
                </div>
              </div>
            );
          })}
        </CardContent>
      </Card>

      {editing && (
        <ProfileDialog
          key={editing.kind === "edit" ? editing.profile.id : "new"}
          editing={editing}
          connected={connected}
          onClose={() => setEditing(null)}
        />
      )}

      <AlertDialog open={deleting !== null} onOpenChange={(open) => !open && setDeleting(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete {deleting?.name}?</AlertDialogTitle>
            <AlertDialogDescription>
              Its lighting and polling rate settings are removed from this PC. The keyboard itself isn't changed.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                const target = deleting;
                setDeleting(null);
                if (target) void run(`delete-${target.id}`, () => deleteProfile(target.id));
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
