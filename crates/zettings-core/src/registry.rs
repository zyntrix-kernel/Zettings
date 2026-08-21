//! Settings registry model and the built-in category seed.
//!
//! The registry is the canonical, data-driven description of every settings
//! surface (Windows reconstruction spec §12/§19). Pages and sections are
//! declared as data; generic templates render them (Phase 6).

use crate::error::ZettingsError;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated category identifier: lowercase slug, `[a-z0-9-]`, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CategoryId(String);

impl CategoryId {
    /// Validates and constructs a category identifier.
    ///
    /// # Errors
    /// Returns [`ZettingsError::InvalidId`] when the slug grammar is violated.
    pub fn parse(raw: &str) -> Result<Self, ZettingsError> {
        validate_slug(raw).map(|()| Self(raw.to_owned()))
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_slug(raw: &str) -> Result<(), ZettingsError> {
    if raw.is_empty() {
        return Err(ZettingsError::invalid_id(raw, "empty"));
    }
    if !raw
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ZettingsError::invalid_id(
            raw,
            "allowed characters are [a-z0-9-]",
        ));
    }
    Ok(())
}

/// The control rendered on the right side of a [`SettingDefinition`] card
/// (Windows reconstruction spec §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlType {
    /// On/off switch.
    Toggle,
    /// Push button / action.
    Button,
    /// Drop-down selection.
    ComboBox,
    /// Continuous numeric range.
    Slider,
    /// Hyperlink to another surface.
    Link,
    /// Read-only status text.
    StatusText,
    /// Navigation affordance into a child page.
    Navigate,
}

/// Requirements a setting places on the environment before it can be used
/// (`PLAN.md` §3 setting contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ValueRequirement {
    /// Changing this value requires elevated authorization (`PolicyKit`).
    pub requires_admin: bool,
    /// The controlling hardware must be present for the setting to apply.
    pub requires_hardware: bool,
    /// Changes take effect after reboot or session restart.
    pub requires_reboot: bool,
}

/// A single addressable setting entity in the settings graph.
///
/// Every navigable setting carries the full contract required by `PLAN.md` §3:
/// identity, search metadata (aliases/keywords), presentation (control type),
/// behavior flags ([`ValueRequirement`]), and its stable route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingDefinition {
    /// Stable, globally unique identifier, e.g. `system.display.scale`.
    pub id: String,
    /// Human-facing title.
    pub title: String,
    /// Supporting description shown under the title.
    pub description: String,
    /// Owning category.
    pub category: CategoryId,
    /// Page within the category, e.g. `display`.
    pub page: String,
    /// Section within the page; sections render in declaration order.
    pub section: String,
    /// Stable deep-link route for this setting's page context.
    pub route: crate::route::RouteId,
    /// Alternative phrasings accepted by search.
    pub aliases: Vec<String>,
    /// Additional search keywords.
    pub keywords: Vec<String>,
    /// Control rendered for this setting.
    pub control_type: ControlType,
    /// Admin/hardware/reboot requirements.
    pub requirements: ValueRequirement,
    /// Search weight boost; higher ranks above equal matches.
    pub search_weight: i32,
}

/// Flattened summary of a top-level category, as consumed by the shell
/// navigation pane (IPC payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Category identifier.
    pub id: CategoryId,
    /// Display title.
    pub title: String,
    /// One-line description for tooltips and Home cards.
    pub description: String,
    /// Icon identifier resolved by the frontend icon system.
    pub icon: String,
    /// Entry route into the category.
    pub route: crate::route::RouteId,
}

/// Identifiers of the twelve built-in categories, in navigation order.
/// Preserves the recognizable Windows Settings mental model mapped onto Linux
/// backends (docs/research/windows-to-zyntrix-mapping.md §1). `home` is a
/// shell surface, not a category, and is therefore not listed here.
pub const BUILT_IN_CATEGORY_IDS: [&str; 12] = [
    "system",
    "devices",
    "network",
    "personalization",
    "apps",
    "accounts",
    "time-language",
    "gaming",
    "accessibility",
    "privacy-security",
    "updates",
    "developer",
];

/// Builds the built-in categories in navigation order.
pub fn built_in_categories() -> IndexMap<CategoryId, CategorySummary> {
    const SEEDS: [(&str, &str, &str, &str); 12] = [
        (
            "system",
            "System",
            "Display, sound, power, storage and notifications.",
            "monitor",
        ),
        (
            "devices",
            "Bluetooth & Devices",
            "Bluetooth, printers, mice, keyboards and cameras.",
            "bluetooth",
        ),
        (
            "network",
            "Network & Internet",
            "Wi-Fi, ethernet, VPN and firewall.",
            "wifi",
        ),
        (
            "personalization",
            "Personalization",
            "Background, colors, themes, fonts and cursor.",
            "palette",
        ),
        (
            "apps",
            "Apps",
            "Installed apps, defaults, startup and packages.",
            "layout-grid",
        ),
        (
            "accounts",
            "Accounts",
            "Users, sign-in and synchronization.",
            "user-round",
        ),
        (
            "time-language",
            "Time & Language",
            "Date and time, region, language and formats.",
            "clock",
        ),
        (
            "gaming",
            "Gaming",
            "Game mode, captures and performance profiles.",
            "gamepad-2",
        ),
        (
            "accessibility",
            "Accessibility",
            "Vision, hearing, interaction and narration.",
            "accessibility",
        ),
        (
            "privacy-security",
            "Privacy & Security",
            "Permissions, firewall and system security.",
            "shield-check",
        ),
        (
            "updates",
            "Updates",
            "System updates and package maintenance.",
            "refresh-cw",
        ),
        (
            "developer",
            "Developer",
            "Developer options, services and diagnostics.",
            "terminal",
        ),
    ];
    debug_assert_eq!(SEEDS.len(), BUILT_IN_CATEGORY_IDS.len());
    SEEDS
        .iter()
        .map(|(id, title, desc, icon)| {
            (
                CategoryId((*id).to_owned()),
                CategorySummary {
                    id: CategoryId((*id).to_owned()),
                    title: (*title).to_owned(),
                    description: (*desc).to_owned(),
                    icon: (*icon).to_owned(),
                    // Infallible by grammar for these seeds; a panic here would
                    // indicate a corrupted seed table at compile time of intent.
                    route: crate::route::RouteId::parse(&format!("zettings://{id}"))
                        .expect("built-in seed routes are valid"),
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_ids_validate_slugs() {
        assert!(CategoryId::parse("time-language").is_ok());
        assert!(CategoryId::parse("").is_err());
        assert!(CategoryId::parse("Bad_ID").is_err());
    }

    #[test]
    fn built_in_seed_matches_declared_ids_and_is_valid() {
        let cats = built_in_categories();
        assert_eq!(cats.len(), BUILT_IN_CATEGORY_IDS.len());
        for id in BUILT_IN_CATEGORY_IDS {
            assert!(
                cats.contains_key(&CategoryId(id.to_owned())),
                "{id} missing from seed"
            );
            assert_eq!(cats[&CategoryId(id.to_owned())].id.as_str(), id);
        }
    }
}
