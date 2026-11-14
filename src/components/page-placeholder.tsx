import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import type { PageInfo } from "@/pages";

// Stand-in for a page whose controls arrive in a later roadmap phase.
export function PagePlaceholder({ page }: { page: PageInfo }) {
  return (
    <Empty className="border border-dashed">
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <page.icon />
        </EmptyMedia>
        <EmptyTitle>{page.title} is on the way</EmptyTitle>
        <EmptyDescription>
          Connect your keyboard to see these settings once this page is built.
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
