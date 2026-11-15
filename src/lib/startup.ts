import { invoke } from "@tauri-apps/api/core";

/** Whether Windows starts Keystra in the tray when you sign in. */
export function getStartWithWindows(): Promise<boolean> {
  return invoke<boolean>("start_with_windows");
}

/** Turns starting with Windows on or off. Resolves to what Windows now has. */
export function setStartWithWindows(enabled: boolean): Promise<boolean> {
  return invoke<boolean>("set_start_with_windows", { enabled });
}
