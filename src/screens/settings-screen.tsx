import { useEffect, useState } from "react";

import { ErrorAlert } from "@/components/notices";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { useAction } from "@/hooks/use-action";
import { getStartWithWindows, setStartWithWindows } from "@/lib/startup";

// App settings: starting with Windows, and how the window and tray behave.
export function SettingsScreen() {
  const [startup, setStartup] = useState<boolean | null>(null);
  const { busy, error, run } = useAction("changing the startup setting");

  useEffect(() => {
    void run("read", async () => setStartup(await getStartWithWindows()));
  }, [run]);

  const toggle = (enabled: boolean) => run("toggle", async () => setStartup(await setStartWithWindows(enabled)));

  return (
    <div className="flex flex-col gap-6">
      <ErrorAlert title="That didn't work" message={error} />

      <Card>
        <CardHeader>
          <CardTitle>Startup</CardTitle>
          <CardDescription>Keep your profile ready from the moment you sign in.</CardDescription>
        </CardHeader>
        <CardContent>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel htmlFor="start-with-windows">Start Keystra with Windows</FieldLabel>
              <FieldDescription>
                Keystra opens quietly in the tray when you sign in and sends your profile to the keyboard.
              </FieldDescription>
            </FieldContent>
            <Switch
              id="start-with-windows"
              checked={startup ?? false}
              disabled={startup === null || busy !== null}
              onCheckedChange={toggle}
            />
          </Field>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>The tray</CardTitle>
          <CardDescription>
            Closing this window keeps Keystra running in the tray, next to the clock. Right-click its icon to switch
            profiles or quit, and click it to open this window again.
          </CardDescription>
        </CardHeader>
      </Card>
    </div>
  );
}
