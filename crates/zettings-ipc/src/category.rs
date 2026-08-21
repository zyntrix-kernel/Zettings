//! Category and registry snapshot payloads.

use serde::{Deserialize, Serialize};
use zettings_core::CategorySummary;
use zettings_core::built_in_categories;

/// Wire representation of a top-level category for the navigation pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "category-summary.ts")]
pub struct CategorySummaryDto {
    /// Category slug identifier.
    #[ts(type = "string")]
    pub id: String,
    /// Display title.
    pub title: String,
    /// One-line description.
    pub description: String,
    /// Icon identifier resolved by the frontend icon system.
    pub icon: String,
    /// Entry deep link (`zettings://...`).
    #[ts(type = "string")]
    pub route: String,
}

impl From<&CategorySummary> for CategorySummaryDto {
    fn from(value: &CategorySummary) -> Self {
        Self {
            id: value.id.as_str().to_owned(),
            title: value.title.clone(),
            description: value.description.clone(),
            icon: value.icon.clone(),
            route: value.route.as_str().to_owned(),
        }
    }
}

/// Snapshot of the top-level settings graph served to the shell at startup.
///
/// The seed graph ships with the binary; plugin-provided categories extend it
/// in later phases (plugin lifecycle, PLAN §1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "registry-snapshot.ts")]
pub struct RegistrySnapshotDto {
    /// Top-level categories in navigation order.
    pub categories: Vec<CategorySummaryDto>,
}

impl RegistrySnapshotDto {
    /// Builds the snapshot from the built-in category seed.
    pub fn built_in() -> Self {
        Self {
            categories: built_in_categories()
                .values()
                .map(CategorySummaryDto::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_serializes_round_trip() {
        let snap = RegistrySnapshotDto::built_in();
        let json = serde_json::to_string(&snap).expect("serialize");
        let back: RegistrySnapshotDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, snap);
        assert_eq!(back.categories.len(), 12);
    }
}
