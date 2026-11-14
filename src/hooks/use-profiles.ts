import { useLiveValue } from "@/hooks/use-live-value";
import { listProfiles, onProfilesChanged, type ProfileList } from "@/lib/profiles";

/** Saved profiles, kept live by the Rust store's change event. `null` until the first load. */
export function useProfiles(): ProfileList | null {
  return useLiveValue(listProfiles, onProfilesChanged, "could not load profiles");
}
