/** A readable message for anything a command or a promise rejects with. */
export function errorText(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
