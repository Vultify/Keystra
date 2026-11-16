import type { ReactNode } from "react";
import { LayersIcon, TriangleAlertIcon } from "lucide-react";

import { KeyboardNotice } from "@/components/keyboard-notice";
import { Alert, AlertAction, AlertDescription, AlertTitle } from "@/components/ui/alert";
import type { KeyboardStatus } from "@/lib/keyboard";
import type { ProfileList } from "@/lib/profiles";

// The one way screens show a failed action: nothing when there's no message.
export function ErrorAlert({ title, message, action }: { title: string; message: string | null; action?: ReactNode }) {
  if (!message) return null;
  return (
    <Alert variant="destructive">
      <AlertTitle>{title}</AlertTitle>
      <AlertDescription>{message}</AlertDescription>
      {action && <AlertAction>{action}</AlertAction>}
    </Alert>
  );
}

// Explains why profiles can't be changed, when the saved file couldn't be used.
export function ReadOnlyNotice({ list }: { list: ProfileList | null }) {
  if (!list?.readOnly) return null;
  return (
    <Alert>
      <TriangleAlertIcon />
      <AlertTitle>Profiles are read-only</AlertTitle>
      <AlertDescription>
        Keystra couldn't use your saved profiles file, so it left the file untouched and turned off changes.
        Reason: {list.readOnly}
      </AlertDescription>
    </Alert>
  );
}

// What the Lighting and Performance pages show when there's no profile to edit.
export function NoProfileNotice({ status, list }: { status: KeyboardStatus | null; list: ProfileList | null }) {
  if (!list) return null;
  return (
    <div className="flex flex-col gap-6">
      <KeyboardNotice status={status} />
      {list.readOnly ? (
        <ReadOnlyNotice list={list} />
      ) : (
        <Alert>
          <LayersIcon />
          <AlertTitle>No profile in use</AlertTitle>
          <AlertDescription>Switch to a profile on the Profiles page to change its settings here.</AlertDescription>
        </Alert>
      )}
    </div>
  );
}
