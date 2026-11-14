import { useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import { PagePlaceholder } from "@/components/page-placeholder";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useKeyboardStatus } from "@/hooks/use-keyboard-status";
import { PAGES, type PageId, type PageInfo } from "@/pages";
import { DiagnosticsScreen } from "@/screens/diagnostics-screen";
import type { KeyboardStatus } from "@/lib/keyboard";

function PageContent({ page, status }: { page: PageInfo; status: KeyboardStatus | null }) {
  if (page.id === "diagnostics") return <DiagnosticsScreen status={status} />;
  return <PagePlaceholder page={page} />;
}

// App layout: sidebar on the left, the selected page on the right.
export default function App() {
  const [activePage, setActivePage] = useState<PageId>("lighting");
  const status = useKeyboardStatus();
  const page = PAGES.find((p) => p.id === activePage) ?? PAGES[0];

  return (
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar activePage={page.id} onNavigate={setActivePage} status={status} />
        <SidebarInset>
          <div className="flex h-svh flex-col gap-6 overflow-y-auto p-8">
            <header className="flex flex-col gap-1">
              <h1 className="text-2xl font-semibold">{page.title}</h1>
              <p className="text-muted-foreground">{page.description}</p>
            </header>
            <PageContent page={page} status={status} />
          </div>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
