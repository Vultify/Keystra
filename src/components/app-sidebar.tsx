import keystraMark from "@/assets/keystra-mark.png";
import { Badge } from "@/components/ui/badge";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { PAGES, type PageId } from "@/pages";

type AppSidebarProps = {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
};

// Left navigation with the brand, page links and the keyboard connection status.
export function AppSidebar({ activePage, onNavigate }: AppSidebarProps) {
  return (
    <Sidebar collapsible="none" className="h-svh border-r">
      <SidebarHeader className="px-4 pt-5 pb-4">
        <div className="flex items-center gap-2.5">
          <img src={keystraMark} alt="" className="size-8" draggable={false} />
          <span className="font-heading text-lg font-semibold tracking-[0.2em]">
            KEYSTRA
          </span>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {PAGES.map((page) => (
                <SidebarMenuItem key={page.id}>
                  <SidebarMenuButton
                    isActive={page.id === activePage}
                    onClick={() => onNavigate(page.id)}
                  >
                    <page.icon />
                    <span>{page.title}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="p-4">
        <Badge variant="outline">Keyboard not connected</Badge>
      </SidebarFooter>
    </Sidebar>
  );
}
