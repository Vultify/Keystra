import { ActivityIcon, GaugeIcon, LayersIcon, SunIcon, type LucideIcon } from "lucide-react";

export type PageId = "lighting" | "performance" | "profiles" | "diagnostics";

export type PageInfo = {
  id: PageId;
  title: string;
  description: string;
  icon: LucideIcon;
};

// Every page in the sidebar, in display order.
export const PAGES: PageInfo[] = [
  {
    id: "lighting",
    title: "Lighting",
    description: "Pick an effect, a color and how bright your keyboard glows.",
    icon: SunIcon,
  },
  {
    id: "performance",
    title: "Performance",
    description: "Set how often your keyboard reports key presses to your PC.",
    icon: GaugeIcon,
  },
  {
    id: "profiles",
    title: "Profiles",
    description: "Save setups and switch between them, or link them to your games.",
    icon: LayersIcon,
  },
  {
    id: "diagnostics",
    title: "Diagnostics",
    description: "See exactly what Keystra reads from your keyboard.",
    icon: ActivityIcon,
  },
];
