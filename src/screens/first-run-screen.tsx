import { GaugeIcon, LayersIcon, SunIcon, type LucideIcon } from "lucide-react";

import keystraMark from "@/assets/keystra-mark.png";
import { KeyboardNotice } from "@/components/keyboard-notice";
import { ErrorAlert } from "@/components/notices";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { useAction } from "@/hooks/use-action";
import type { KeyboardStatus } from "@/lib/keyboard";
import { applyProfile, createProfileFromKeyboard, saveKeyboardToProfile, type ProfileList } from "@/lib/profiles";

const BASICS: { icon: LucideIcon; title: string; text: string }[] = [
  { icon: SunIcon, title: "Lighting", text: "Pick an effect and a color. The keyboard changes as you go." },
  { icon: GaugeIcon, title: "Performance", text: "Choose how often the keyboard reports your key presses." },
  { icon: LayersIcon, title: "Profiles", text: "Save a few setups and switch between them in one click." },
];

// Shown until a profile is in use: finds the keyboard, explains the basics, and starts a profile
// from the keyboard's own settings so nothing on it changes and no saved profile is overwritten.
export function FirstRunScreen({ status, list }: { status: KeyboardStatus | null; list: ProfileList }) {
  const { busy, error, run } = useAction("first-run setup");
  const connected = status?.state === "connected";

  // A brand-new install has only the untouched Default, which takes the keyboard's settings.
  // With saved profiles already there, the keyboard's settings go into a new one instead.
  function start() {
    void run("start", async () => {
      const only = list.profiles.length === 1 ? list.profiles[0] : null;
      const profile = only ? await saveKeyboardToProfile(only.id) : await createProfileFromKeyboard();
      await applyProfile(profile.id);
    });
  }

  return (
    <div className="mx-auto flex w-full max-w-xl flex-col gap-6 py-6">
      <Card>
        <CardHeader className="items-center text-center">
          <img src={keystraMark} alt="" className="mx-auto mb-2 size-14" draggable={false} />
          <CardTitle className="text-2xl">Welcome to Keystra</CardTitle>
          <CardDescription>
            {connected ? "Found your keyboard. Here's how Keystra works." : "First, let's find your keyboard."}
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          {connected ? (
            BASICS.map(({ icon: Icon, title, text }) => (
              <div key={title} className="flex items-start gap-3">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/15 text-primary">
                  <Icon />
                </span>
                <div className="flex flex-col">
                  <span className="font-medium">{title}</span>
                  <span className="text-sm text-muted-foreground">{text}</span>
                </div>
              </div>
            ))
          ) : (
            <KeyboardNotice status={status} />
          )}
          <ErrorAlert title="Couldn't get started" message={error} />
        </CardContent>
        <CardFooter className="flex-col gap-2">
          <Button className="w-full" disabled={!connected || busy !== null || list.readOnly !== null} onClick={start}>
            {busy && <Spinner data-icon="inline-start" />}
            Get started
          </Button>
          <p className="text-center text-xs text-muted-foreground">
            Your profile starts with the keyboard's current settings, so nothing on it changes.
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}
