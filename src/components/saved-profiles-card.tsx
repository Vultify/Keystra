import { useState, type FormEvent } from "react";
import { DownloadIcon, PencilIcon, PlusIcon, SendIcon, Trash2Icon, TriangleAlertIcon } from "lucide-react";

import { ProfileEditorDialog } from "@/components/profile-editor-dialog";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { onboardSlotLabel } from "@/lib/keyboard";
import { describeLighting } from "@/lib/lighting";
import { logError } from "@/lib/logging";
import {
  applyProfile,
  createProfile,
  createProfileFromKeyboard,
  deleteProfile,
  MAX_NAME_LEN,
  saveKeyboardToProfile,
  saveProfile,
  type Profile,
  type ProfileList,
} from "@/lib/profiles";

function lightingText({ lighting }: Profile): string {
  const { effect, option, color } = describeLighting(lighting.mode, lighting.rainbow, lighting.color, lighting.direction);
  return [effect, option?.value, color].filter(Boolean).join(", ");
}

// Test bench for profiles: add, edit, apply to the keyboard, refresh from it, and delete. The real
// Profiles page replaces it.
export function SavedProfilesCard({ connected, list }: { connected: boolean; list: ProfileList | null }) {
  const profiles = list?.profiles;
  const readOnly = list?.readOnly ?? null;
  const [name, setName] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<Profile | null>(null);
  const locked = busy !== null || readOnly !== null;

  async function run(target: string, action: () => Promise<unknown>) {
    setBusy(target);
    setError(null);
    try {
      await action();
    } catch (err) {
      setError(String(err));
      logError("profile action failed", err);
    } finally {
      setBusy(null);
    }
  }

  function addProfile(event: FormEvent) {
    event.preventDefault();
    void run("new", async () => {
      await (connected ? createProfileFromKeyboard(name.trim()) : createProfile(name.trim()));
      setName("");
    });
  }

  function apply(id: number) {
    void run(`apply-${id}`, () => applyProfile(id));
  }

  // A failed save stays in the dialog so no edits are lost; a save that worked but didn't reach
  // the keyboard closes it and says so here.
  async function saveEdits(profile: Profile) {
    const { applyError } = await saveProfile(profile);
    setError(applyError && `Saved, but the keyboard didn't take it: ${applyError}`);
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Saved profiles</CardTitle>
        <CardDescription>
          Stored on this PC. Apply sends a profile to the keyboard; Save keyboard copies the keyboard into it.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <form onSubmit={addProfile}>
          <Field orientation="horizontal">
            <FieldLabel htmlFor="new-profile-name" className="sr-only">
              New profile name
            </FieldLabel>
            <Input
              id="new-profile-name"
              placeholder="New profile name"
              value={name}
              maxLength={MAX_NAME_LEN}
              onChange={(event) => setName(event.target.value)}
            />
            <Button type="submit" disabled={name.trim() === "" || locked}>
              {busy === "new" ? <Spinner data-icon="inline-start" /> : <PlusIcon data-icon="inline-start" />}
              {connected ? "Save keyboard as new profile" : "Add profile"}
            </Button>
          </Field>
        </form>

        {readOnly && (
          <Alert>
            <TriangleAlertIcon />
            <AlertTitle>Profiles are read-only</AlertTitle>
            <AlertDescription>
              Keystra couldn't use your saved profiles file, so it left the file untouched and turned off
              changes. Reason: {readOnly}
            </AlertDescription>
          </Alert>
        )}

        {error && (
          <Alert variant="destructive">
            <AlertTitle>That didn't work</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {profiles && (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Onboard</TableHead>
                <TableHead>Lighting</TableHead>
                <TableHead>Polling</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {profiles.map((profile) => (
                <TableRow key={profile.id}>
                  <TableCell className="font-medium">
                    <span className="flex items-center gap-2">
                      {profile.name}
                      {list?.activeId === profile.id && <Badge variant="secondary">Active</Badge>}
                    </span>
                  </TableCell>
                  <TableCell>{onboardSlotLabel(profile.onboardSlot)}</TableCell>
                  <TableCell>{lightingText(profile)}</TableCell>
                  <TableCell>{profile.pollingRateHz} Hz</TableCell>
                  <TableCell>
                    <div className="flex justify-end gap-1">
                      <Button size="sm" disabled={!connected || locked} onClick={() => apply(profile.id)}>
                        {busy === `apply-${profile.id}` ? (
                          <Spinner data-icon="inline-start" />
                        ) : (
                          <SendIcon data-icon="inline-start" />
                        )}
                        Apply
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={`Edit ${profile.name}`}
                        disabled={locked}
                        onClick={() => setEditing(profile)}
                      >
                        <PencilIcon />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={`Save keyboard into ${profile.name}`}
                        title="Save keyboard into this profile"
                        disabled={!connected || locked}
                        onClick={() => run(`save-${profile.id}`, () => saveKeyboardToProfile(profile.id))}
                      >
                        {busy === `save-${profile.id}` ? <Spinner /> : <DownloadIcon />}
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={`Delete ${profile.name}`}
                        disabled={profiles.length === 1 || locked}
                        onClick={() => run(`delete-${profile.id}`, () => deleteProfile(profile.id))}
                      >
                        <Trash2Icon />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}

        {editing && (
          <ProfileEditorDialog key={editing.id} profile={editing} onClose={() => setEditing(null)} onSave={saveEdits} />
        )}
      </CardContent>
    </Card>
  );
}
