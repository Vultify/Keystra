import { useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useKeyboardSettings } from "@/hooks/use-keyboard-settings";
import { useKeyboardStatus } from "@/hooks/use-keyboard-status";
import { useProfiles } from "@/hooks/use-profiles";
import { PAGES, type PageId } from "@/pages";
import { DiagnosticsScreen } from "@/screens/diagnostics-screen";
import { FirstRunScreen } from "@/screens/first-run-screen";
import { LightingScreen } from "@/screens/lighting-screen";
import { PerformanceScreen } from "@/screens/performance-screen";
import { ProfilesScreen } from "@/screens/profiles-screen";

// App layout: sidebar on the left, the selected page on the right. Until a profile is in use, the
// first-run screen takes the page's place.
export default function App() {
  const [activePage, setActivePage] = useState<PageId>("lighting");
  const status = useKeyboardStatus();
  const settings = useKeyboardSettings(status);
  const list = useProfiles();
  const page = PAGES.find((p) => p.id === activePage) ?? PAGES[0];
  const firstRun = list !== null && list.activeId === null && list.readOnly === null && page.id !== "diagnostics";
  const activeProfile = list?.profiles.find((profile) => profile.id === list.activeId) ?? null;

  return (
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar activePage={page.id} onNavigate={setActivePage} status={status} activeProfile={activeProfile} />
        <SidebarInset>
          <div className="flex h-svh flex-col gap-6 overflow-y-auto p-8">
            {firstRun ? (
              <FirstRunScreen status={status} list={list} />
            ) : (
              <>
                <header className="flex flex-col gap-1">
                  <h1 className="text-2xl font-semibold">{page.title}</h1>
                  <p className="text-muted-foreground">{page.description}</p>
                </header>
                {page.id === "lighting" && <LightingScreen status={status} list={list} />}
                {page.id === "performance" && <PerformanceScreen status={status} list={list} />}
                {page.id === "profiles" && <ProfilesScreen status={status} list={list} />}
                {page.id === "diagnostics" && <DiagnosticsScreen status={status} settings={settings} list={list} />}
              </>
            )}
          </div>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
