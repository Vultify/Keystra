import { useCallback, useEffect, useState, type ReactNode } from "react";
import { RefreshCwIcon, TriangleAlertIcon, UsbIcon } from "lucide-react";

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
  LEVEL_MAX,
  readKeyboard,
  toHex,
  type KeyboardStatus,
  type Snapshot,
} from "@/lib/keyboard";
import { logError } from "@/lib/logging";

type Row = { label: string; value: ReactNode };

function settingsRows(snapshot: Snapshot): Row[] {
  const { lighting } = snapshot;
  return [
    { label: "Profile", value: `Profile ${snapshot.profile + 1}` },
    {
      label: "Polling rate",
      value: snapshot.pollingRate ? `${snapshot.pollingRate} Hz` : "Unknown",
    },
    {
      label: "Lighting effect",
      value: `${lighting.modeName ?? "Unknown"} (mode ${lighting.modeId})`,
    },
    { label: "Brightness", value: `${lighting.brightness} of ${LEVEL_MAX}` },
    { label: "Speed", value: `${lighting.speed} of ${LEVEL_MAX}` },
    { label: "Color", value: <LightingColor lighting={lighting} /> },
    { label: "Direction", value: String(lighting.direction) },
  ];
}

function LightingColor({ lighting }: { lighting: Snapshot["lighting"] }) {
  if (lighting.rainbow) return "Rainbow";
  if (lighting.color === null) return <span className="text-muted-foreground">Set by the effect</span>;
  return (
    <span className="flex items-center gap-2">
      <span className="size-4 rounded-sm border" style={{ backgroundColor: lighting.color }} />
      {lighting.color}
    </span>
  );
}

// Shows what Keystra reads from the keyboard, parsed and as raw bytes, to compare against MagCrate.
export function DiagnosticsScreen({ status }: { status: KeyboardStatus | null }) {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [reading, setReading] = useState(false);
  const [readError, setReadError] = useState<string | null>(null);

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
      {connected && status.magcrateOpen && (
        <Alert>
          <TriangleAlertIcon />
          <AlertTitle>MagCrate is running</AlertTitle>
          <AlertDescription>
            MagCrate and Keystra share the same line to your keyboard, so their replies can get mixed up. Close
            MagCrate, including its icon in the system tray, while you use Keystra.
          </AlertDescription>
        </Alert>
      )}

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
                  {settingsRows(snapshot).map((row) => (
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
                  <TableRow key={exchange.command}>
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
