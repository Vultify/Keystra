import { KeyboardNotice } from "@/components/keyboard-notice";
import { ErrorAlert, NoProfileNotice } from "@/components/notices";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useActiveProfile } from "@/hooks/use-active-profile";
import type { KeyboardStatus } from "@/lib/keyboard";
import { POLLING_RATES, type ProfileList } from "@/lib/profiles";

/** What each polling rate is good for, in plain words. */
function pollingHint(hz: number): string {
  if (hz >= 4000) return "Fastest response, for competitive games. Uses a little more of your PC's processor.";
  if (hz >= 1000) return "Very responsive and light on your PC. A great everyday choice.";
  return "Slower reporting. Only useful for older PCs or to save battery.";
}

// Edits the active profile's polling rate; changes reach the keyboard as soon as they're picked.
export function PerformanceScreen({ status, list }: { status: KeyboardStatus | null; list: ProfileList | null }) {
  const { profile, commit, saving, problem } = useActiveProfile(list);
  if (!profile) return <NoProfileNotice status={status} list={list} />;

  return (
    <div className="flex flex-col gap-6">
      <KeyboardNotice status={status} />
      <ErrorAlert title="That change didn't go through" message={problem} />

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Polling rate
            {saving && <Spinner />}
          </CardTitle>
          <CardDescription>
            How many times a second your keyboard tells your PC which keys are pressed. Higher means your
            presses arrive sooner. Editing <span className="text-foreground">{profile.name}</span>.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <ToggleGroup
            variant="outline"
            className="flex-wrap"
            value={[String(profile.pollingRateHz)]}
            onValueChange={(values) => {
              const hz = values[0];
              if (hz) commit({ ...profile, pollingRateHz: Number(hz) });
            }}
          >
            {POLLING_RATES.map((hz) => (
              <ToggleGroupItem key={hz} value={String(hz)}>
                {hz} Hz
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <p className="text-sm text-muted-foreground">{pollingHint(profile.pollingRateHz)}</p>
        </CardContent>
      </Card>
    </div>
  );
}
