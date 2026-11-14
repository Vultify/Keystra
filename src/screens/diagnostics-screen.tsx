import type { ReactNode } from "react";
import { RefreshCwIcon } from "lucide-react";

import { KeyboardNotice } from "@/components/keyboard-notice";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { KeyboardSettings } from "@/hooks/use-keyboard-settings";
import { formatFirmware, MAX_LEVEL, onboardSlotLabel, toHex, type KeyboardStatus, type Snapshot } from "@/lib/keyboard";
import { describeLighting } from "@/lib/lighting";
import type { ProfileList } from "@/lib/profiles";

type Row = { label: string; value: ReactNode };

/** "Vultify (Profile 2)" when the active Keystra profile uses the keyboard's onboard profile. */
function profileLabel(snapshot: Snapshot, list: ProfileList | null): string {
  const slot = onboardSlotLabel(snapshot.onboardSlot);
  const active = list?.profiles.find((profile) => profile.id === list.activeId);
  return active && active.onboardSlot === snapshot.onboardSlot ? `${active.name} (${slot})` : slot;
}

function settingsRows(snapshot: Snapshot, list: ProfileList | null): Row[] {
  const { lighting, info } = snapshot;
  const described = describeLighting(lighting.modeId, lighting.rainbow, lighting.color, lighting.direction);
  return [
    { label: "Model", value: info.model ?? "Unknown" },
    { label: "Device ID", value: info.deviceId },
    { label: "Firmware", value: formatFirmware(info.firmwareVersion) },
    { label: "Profile", value: profileLabel(snapshot, list) },
    { label: "Polling rate", value: snapshot.pollingRate ? `${snapshot.pollingRate} Hz` : "Unknown" },
    { label: "Lighting effect", value: described.effect },
    { label: "Brightness", value: `${lighting.brightness} of ${MAX_LEVEL}` },
    { label: "Speed", value: `${lighting.speed} of ${MAX_LEVEL}` },
    { label: "Color", value: <LightingColor color={described.color} /> },
    ...(described.option ? [{ label: described.option.label, value: described.option.value }] : []),
  ];
}

function LightingColor({ color }: { color: string | null }) {
  if (color === null) return <span className="text-muted-foreground">Set by the effect</span>;
  if (!color.startsWith("#")) return color;
  return (
    <span className="flex items-center gap-2">
      <span className="size-4 rounded-sm border" style={{ backgroundColor: color }} />
      {color}
    </span>
  );
}

// Shows exactly what Keystra reads from the keyboard, parsed and as raw bytes, for checking
// against MagCrate and the protocol notes.
export function DiagnosticsScreen({
  status,
  settings,
  list,
}: {
  status: KeyboardStatus | null;
  settings: KeyboardSettings;
  list: ProfileList | null;
}) {
  const { snapshot, reading, error, refresh } = settings;
  const connected = status?.state === "connected";

  return (
    <div className="flex flex-col gap-6">
      <KeyboardNotice status={status} showHelper />

      {connected && (
        <Card>
          <CardHeader>
            <CardTitle>Current settings</CardTitle>
            <CardDescription>Read straight from the keyboard. Nothing here changes a setting.</CardDescription>
            <CardAction>
              <Button variant="outline" onClick={refresh} disabled={reading}>
                {reading ? <Spinner data-icon="inline-start" /> : <RefreshCwIcon data-icon="inline-start" />}
                Read again
              </Button>
            </CardAction>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            {error && (
              <Alert variant="destructive">
                <AlertTitle>Couldn't read the keyboard</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            )}
            {snapshot && (
              <Table>
                <TableBody>
                  {settingsRows(snapshot, list).map((row) => (
                    <TableRow key={row.label}>
                      <TableCell className="w-48 text-muted-foreground">{row.label}</TableCell>
                      <TableCell>{row.value}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      )}

      {connected && snapshot && (
        <Card>
          <CardHeader>
            <CardTitle>Raw replies</CardTitle>
            <CardDescription>The exact bytes sent and received, for checking against the protocol notes.</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-40">Command</TableHead>
                  <TableHead>Sent</TableHead>
                  <TableHead>Received</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {snapshot.exchanges.map((exchange) => (
                  <TableRow key={exchange.name}>
                    <TableCell className="font-medium">{exchange.name}</TableCell>
                    <TableCell className="font-mono text-xs whitespace-normal">{toHex(exchange.request)}</TableCell>
                    <TableCell className="font-mono text-xs whitespace-normal break-all">{toHex(exchange.reply)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
