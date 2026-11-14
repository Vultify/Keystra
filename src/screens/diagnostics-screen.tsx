import { useCallback, useEffect, useState, type ReactNode } from "react";
import { InfoIcon, RefreshCwIcon, TriangleAlertIcon, UsbIcon } from "lucide-react";

import { SavedProfilesCard } from "@/components/saved-profiles-card";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import {
  formatFirmware,
  MAX_LEVEL,
  onboardSlotLabel,
  onKeyboardSettings,
  readKeyboard,
  toHex,
  type KeyboardStatus,
  type MagCrateState,
  type Snapshot,
} from "@/lib/keyboard";
import { useProfiles } from "@/hooks/use-profiles";
import { describeLighting } from "@/lib/lighting";
import { logError } from "@/lib/logging";
import type { ProfileList } from "@/lib/profiles";

type Row = { label: string; value: ReactNode };

/** "Vultify (Profile 2)" when the active Keystra profile uses the keyboard's onboard profile. */
function profileLabel(snapshot: Snapshot, list: ProfileList | null): string {
  const slot = onboardSlotLabel(snapshot.onboardSlot);
  const active = list?.profiles.find((profile) => profile.id === list.activeId);
  return active && active.onboardSlot === snapshot.onboardSlot ? `${active.name} (${slot})` : slot;
}

function settingsRows(snapshot: Snapshot, list: ProfileList | null): Row[] {
  const { lighting } = snapshot;
  const described = describeLighting(lighting.modeId, lighting.rainbow, lighting.color, lighting.direction);
  return [
    { label: "Profile", value: profileLabel(snapshot, list) },
    {
      label: "Polling rate",
      value: snapshot.pollingRate ? `${snapshot.pollingRate} Hz` : "Unknown",
    },
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

// Shows what Keystra reads from the keyboard, parsed and as raw bytes, to compare against MagCrate.
export function DiagnosticsScreen({ status }: { status: KeyboardStatus | null }) {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [reading, setReading] = useState(false);
  const [readError, setReadError] = useState<string | null>(null);

  const profiles = useProfiles();
  const connected = status?.state === "connected";
  const deviceId = connected ? status.info.deviceId : null;

  const read = useCallback(async () => {
    setReading(true);
    setReadError(null);
    try {
      setSnapshot(await readKeyboard());
    } catch (err) {
      setReadError(String(err));
      logError("reading the keyboard failed", err);
    } finally {
      setReading(false);
    }
  }, []);

  // Keystra announces the keyboard's settings whenever it changes them, from any screen or the tray.
  useEffect(() => {
    const unlisten = onKeyboardSettings(setSnapshot);
    return () => {
      unlisten.then((stop) => stop()).catch(() => {});
    };
  }, []);

  // Read once each time the keyboard connects.
  useEffect(() => {
    if (deviceId === null) {
      setSnapshot(null);
      return;
    }
    void read();
  }, [deviceId, read]);

  return (
    <div className="flex flex-col gap-6">
      {connected && <MagCrateNotice state={status.magcrate} />}

      <ConnectionCard status={status} />

      {connected && (
        <Card>
          <CardHeader>
            <CardTitle>Current settings</CardTitle>
            <CardDescription>Read straight from the keyboard. Nothing here changes a setting.</CardDescription>
            <CardAction>
              <Button variant="outline" onClick={read} disabled={reading}>
                {reading ? <Spinner data-icon="inline-start" /> : <RefreshCwIcon data-icon="inline-start" />}
                Read again
              </Button>
            </CardAction>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            {readError && (
              <Alert variant="destructive">
                <AlertTitle>Couldn't read the keyboard</AlertTitle>
                <AlertDescription>{readError}</AlertDescription>
              </Alert>
            )}
            {snapshot && (
              <Table>
                <TableBody>
                  {settingsRows(snapshot, profiles).map((row) => (
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

      <SavedProfilesCard connected={connected} list={profiles} />

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
                    <TableCell className="font-mono text-xs whitespace-normal break-all">
                      {toHex(exchange.reply)}
                    </TableCell>
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

// MagCrate shares the keyboard's single reply slot. The app itself is a real risk; its leftover
// background helper usually sits idle, so it gets a softer note.
function MagCrateNotice({ state }: { state: MagCrateState }) {
  if (state === "open") {
    return (
      <Alert>
        <TriangleAlertIcon />
        <AlertTitle>MagCrate is running</AlertTitle>
        <AlertDescription>
          MagCrate and Keystra share the same line to your keyboard, so their replies can get mixed up. Close
          MagCrate, including its icon in the system tray, while you use Keystra.
        </AlertDescription>
      </Alert>
    );
  }
  if (state === "helperOnly") {
    return (
      <Alert>
        <InfoIcon />
        <AlertTitle>MagCrate's background helper is still running</AlertTitle>
        <AlertDescription>
          It stays behind after MagCrate closes and is usually harmless. If readings ever look wrong, end
          iot_driver in Task Manager. MagCrate starts it again when you open MagCrate.
        </AlertDescription>
      </Alert>
    );
  }
  return null;
}

function ConnectionCard({ status }: { status: KeyboardStatus | null }) {
  if (status?.state === "connected") {
    const { info } = status;
    return (
      <Card>
        <CardHeader>
          <CardTitle>{info.model}</CardTitle>
          <CardDescription>Connected by USB cable.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableBody>
              <TableRow>
                <TableCell className="w-48 text-muted-foreground">Device ID</TableCell>
                <TableCell>{info.deviceId}</TableCell>
              </TableRow>
              <TableRow>
                <TableCell className="w-48 text-muted-foreground">Firmware</TableCell>
                <TableCell>{formatFirmware(info.firmwareVersion)}</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    );
  }

  if (status?.state === "unsupported") {
    return (
      <Alert variant="destructive">
        <AlertTitle>Unsupported device</AlertTitle>
        <AlertDescription>
          A Cherry device answered with ID {status.deviceId}, which isn't the MX 8.2 Pro TMR. Keystra won't talk to it.
        </AlertDescription>
      </Alert>
    );
  }

  if (status?.state === "error") {
    return (
      <Alert variant="destructive">
        <AlertTitle>Couldn't reach the keyboard</AlertTitle>
        <AlertDescription>{status.message}</AlertDescription>
      </Alert>
    );
  }

  return (
    <Alert>
      <UsbIcon />
      <AlertTitle>{status === null ? "Looking for your keyboard" : "Keyboard not connected"}</AlertTitle>
      <AlertDescription>
        Plug in your MX 8.2 Pro TMR with the USB cable. Wireless isn't supported yet.
      </AlertDescription>
    </Alert>
  );
}
