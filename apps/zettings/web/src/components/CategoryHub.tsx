/**
 * Category hub (spec §14 Template A): page title, description, then the
 * category's settings areas as cards. Live areas navigate to their L2
 * page; planned areas render as intentional, explained "arriving" rows
 * naming the backend integration that unlocks them (spec §15 — disabled
 * with explanation, never an error-looking dead end).
 */
import type { ReactNode } from "react";
import { CATEGORY_AREAS } from "../lib/pages";
import { categoryIcon } from "../lib/category-icons";
import { SettingsCard } from "./zdl";

export interface CategoryHubProps {
  /** Registry category id. */
  category: string;
  /** Hub title (registry). */
  title: string;
  /** Hub description (registry). */
  description: string;
  /** Navigates to a live L2 area. */
  onOpenArea: (slug: string) => void;
}

export function CategoryHub({ category, title, description, onOpenArea }: CategoryHubProps): ReactNode {
  const areas = CATEGORY_AREAS[category] ?? [];

  return (
    <>
      <h1 tabIndex={-1} className="zdl-page-title">
        {title}
      </h1>
      <p className="zdl-page-description">{description}</p>
      <div className="zdl-card-grid">
        {areas.map((area) => {
          const Icon = categoryIcon(area.slug);
          const live = area.live === true;
          return (
            <SettingsCard
              key={area.slug}
              title={area.title}
              description={area.description}
              icon={<Icon size={20} />}
              {...(live
                ? { onActivate: () => onOpenArea(area.slug) }
                : {
                    control: (
                      <span className="zdl-area-status">
                        Arriving with {area.via}
                      </span>
                    ),
                  })}
            />
          );
        })}
      </div>
    </>
  );
}
