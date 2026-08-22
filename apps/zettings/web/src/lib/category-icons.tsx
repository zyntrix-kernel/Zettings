/**
 * Category iconography (DESIGN.md §7): Lucide stroke icons mapped per
 * registry category id. Icons are decorative — accessible names come from
 * the visible labels; every icon renders with currentColor.
 */
import type { ComponentType } from "react";
import {
  Laptop,
  Bluetooth,
  Wifi,
  Palette,
  LayoutGrid,
  UserRound,
  Languages,
  Gamepad2,
  Accessibility,
  ShieldCheck,
  RefreshCcw,
  Terminal,
  House,
} from "lucide-react";

export const CATEGORY_ICONS: Readonly<Record<string, ComponentType<{ size?: number | string }>>> = {
  system: Laptop,
  devices: Bluetooth,
  network: Wifi,
  personalization: Palette,
  apps: LayoutGrid,
  accounts: UserRound,
  "time-language": Languages,
  gaming: Gamepad2,
  accessibility: Accessibility,
  "privacy-security": ShieldCheck,
  updates: RefreshCcw,
  developer: Terminal,
};

export function categoryIcon(id: string, fallback: ComponentType<{ size?: number | string }> = House) {
  return CATEGORY_ICONS[id] ?? fallback;
}
