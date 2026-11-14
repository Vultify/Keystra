import { useState, type FormEvent } from "react";
import { DownloadIcon, PlusIcon, Trash2Icon, TriangleAlertIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "@/components/ui/spinner";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { useProfiles } from "@/hooks/use-profiles";
import { onboardSlotLabel } from "@/lib/keyboard";
import { lightingModeName, lightingUsesColor } from "@/lib/lighting";
import { logError } from "@/lib/logging";
import {
  createProfile,
  createProfileFromKeyboard,
  deleteProfile,
  MAX_NAME_LEN,
  saveKeyboardToProfile,
  type Profile,
} from "@/lib/profiles";

function describeLighting({ lighting }: Profile): string {
  const name = lightingModeName(lighting.mode);
  if (lighting.rainbow) return `${name}, rainbow`;
  return lightingUsesColor(lighting.mode) ? `${name}, ${lighting.color}` : name;
}

// Test bench for the profile store: add, refresh from the keyboard, and delete profiles. Saving
// reads the keyboard but never changes it.
export function SavedProfilesCard({ connected }: { connected: boolean }) {
  const list = useProfiles();
  const profiles = list?.profiles;
  const readOnly = list?.readOnly ?? null;
  const [name, setName] = useState("");
  const [busy, setBusy] = useState<number | "new" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const locked = busy !== null || readOnly !== null;

  async function run(target: number | "new", action: () => Promise<unknown>) {
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

  return (
    <Card>
      <CardHeader>
        <CardTitle>Saved profiles</CardTitle>
        <CardDescription>
          Stored on this PC. Saving reads the keyboard but doesn't change it.
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
                  <TableCell className="font-medium">{profile.name}</TableCell>
                  <TableCell>{onboardSlotLabel(profile.onboardSlot)}</TableCell>
                  <TableCell>{describeLighting(profile)}</TableCell>
                  <TableCell>{profile.pollingRateHz} Hz</TableCell>
                  <TableCell>
                    <div className="flex justify-end gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={!connected || locked}
                        onClick={() => run(profile.id, () => saveKeyboardToProfile(profile.id))}
                      >
                        {busy === profile.id ? (
                          <Spinner data-icon="inline-start" />
                        ) : (
                          <DownloadIcon data-icon="inline-start" />
                        )}
                        Save keyboard
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={`Delete ${profile.name}`}
                        disabled={profiles.length === 1 || locked}
                        onClick={() => run(profile.id, () => deleteProfile(profile.id))}
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
      </CardContent>
    </Card>
  );
}
