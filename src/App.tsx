import { useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import { PagePlaceholder } from "@/components/page-placeholder";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { PAGES, type PageId } from "@/pages";

// App layout: sidebar on the left, the selected page on the right.
export default function App() {
  const [activePage, setActivePage] = useState<PageId>("lighting");
  const page = PAGES.find((p) => p.id === activePage) ?? PAGES[0];

  return (
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar activePage={page.id} onNavigate={setActivePage} />
        <SidebarInset>
          <div className="flex h-svh flex-col gap-6 overflow-y-auto p-8">
            <header className="flex flex-col gap-1">
              <h1 className="text-2xl font-semibold">{page.title}</h1>
              <p className="text-muted-foreground">{page.description}</p>
            </header>
            <PagePlaceholder page={page} />
          </div>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
