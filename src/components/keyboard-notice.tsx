import { InfoIcon, MonitorIcon, RefreshCwIcon, TriangleAlertIcon, UsbIcon } from "lucide-react";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import type { KeyboardStatus } from "@/lib/keyboard";

// One place for "keyboard not found", "keyboard is busy" and MagCrate warnings, so every page
// explains the same problem the same way. The idle helper note only shows where asked, on Diagnostics.
export function KeyboardNotice({ status, showHelper = false }: { status: KeyboardStatus | null; showHelper?: boolean }) {
  if (status === null || status.state === "disconnected") {
    return (
      <Alert>
        <UsbIcon />
        <AlertTitle>{status === null ? "Looking for your keyboard" : "Keyboard not found"}</AlertTitle>
        <AlertDescription>
          Plug in your MX 8.2 Pro TMR with the USB cable. Wireless isn't supported yet. Changes you make now
          are saved, and Keystra sends the profile in use as soon as the keyboard is back.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.state === "unsupported") {
    return (
      <Alert variant="destructive">
        <AlertTitle>This keyboard isn't supported</AlertTitle>
        <AlertDescription>
          A Cherry device answered with ID {status.deviceId}, which isn't the MX 8.2 Pro TMR, so Keystra leaves it alone.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.state === "installedRunning") {
    return (
      <Alert>
        <MonitorIcon />
        <AlertTitle>The installed Keystra has the keyboard</AlertTitle>
        <AlertDescription>
          This development build leaves the keyboard alone while the installed Keystra runs, so the two don't undo
          each other. Quit Keystra from its tray icon and this build takes over within a few seconds.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.state === "retrying") {
    return (
      <Alert variant="destructive">
        <RefreshCwIcon />
        <AlertTitle>The keyboard stopped answering</AlertTitle>
        <AlertDescription>
          It failed over USB a few times in a row, so Keystra is waiting {status.seconds} seconds before trying again.
          If it keeps happening, try another USB port or cable.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.state === "error") {
    return (
      <Alert variant="destructive">
        <AlertTitle>Keyboard is busy</AlertTitle>
        <AlertDescription>
          Keystra can see your keyboard but couldn't talk to it ({status.message}). Close MagCrate if it's open,
          then unplug the keyboard and plug it back in.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.magcrate === "open") {
    return (
      <Alert>
        <TriangleAlertIcon />
        <AlertTitle>MagCrate is running</AlertTitle>
        <AlertDescription>
          MagCrate and Keystra share the same line to your keyboard, so changes can get mixed up or not stick.
          Close MagCrate, including its icon in the system tray, while you use Keystra.
        </AlertDescription>
      </Alert>
    );
  }
  if (status.magcrate === "helperOnly" && showHelper) {
    return (
      <Alert>
        <InfoIcon />
        <AlertTitle>MagCrate's background helper is still running</AlertTitle>
        <AlertDescription>
          It stays behind after MagCrate closes and is usually harmless. If a change doesn't stick, end iot_driver
          in Task Manager. MagCrate starts it again when you open MagCrate.
        </AlertDescription>
      </Alert>
    );
  }
  return null;
}
