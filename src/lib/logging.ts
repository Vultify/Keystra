import { error as writeError } from "@tauri-apps/plugin-log";

/** Writes a screen-side error to the app log file. Never throws. */
export function logError(message: string, cause?: unknown): void {
  const detail = cause instanceof Error ? (cause.stack ?? cause.message) : cause;
  const line = detail === undefined ? message : `${message}: ${String(detail)}`;
  console.error(line);
  writeError(line).catch(() => {});
}

/** Sends uncaught errors and rejected promises to the log file. */
export function installGlobalErrorLogging(): void {
  window.addEventListener("error", (event) => logError("uncaught error", event.error ?? event.message));
  window.addEventListener("unhandledrejection", (event) => logError("unhandled rejection", event.reason));
}
