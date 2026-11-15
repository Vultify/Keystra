import { ActivityIcon, GaugeIcon, LayersIcon, SettingsIcon, SunIcon, type LucideIcon } from "lucide-react";

export type PageId = "lighting" | "performance" | "profiles" | "settings" | "diagnostics";

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
    description: "Save setups and switch between them.",
    icon: LayersIcon,
  },
  {
    id: "settings",
    title: "Settings",
    description: "Choose how Keystra starts and runs.",
    icon: SettingsIcon,
  },
  {
    id: "diagnostics",
    title: "Diagnostics",
    description: "See exactly what Keystra reads from your keyboard.",
    icon: ActivityIcon,
  },
];
