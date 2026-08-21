//! Search request/response payloads.

use serde::{Deserialize, Serialize};
use zettings_core::built_in_page_definitions;
use zettings_search::{SearchHit, rank_settings};

/// One ranked search result on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "search-hit.ts")]
pub struct SearchHitDto {
    /// Stable setting/page identifier.
    pub setting_id: String,
    /// Display title.
    pub title: String,
    /// Supporting description.
    pub description: String,
    /// Owning category slug.
    #[ts(type = "string")]
    pub category: String,
    /// Deep link for direct navigation.
    #[ts(type = "string")]
    pub route: String,
    /// Computed relevance score (exposed for debugging/UX tie-breaks).
    pub score: i32,
}

/// Response for the `search_registry` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "search-response.ts")]
pub struct SearchResponseDto {
    /// Normalized query that produced these hits.
    pub query: String,
    /// Ranked hits, best first; empty when nothing matched.
    pub hits: Vec<SearchHitDto>,
}

impl SearchResponseDto {
    /// Ranks the built-in registry pages against `raw_query`.
    ///
    /// The seed graph is compiled in; plugin-provided definitions extend the
    /// searched set once the module loader lands. Usage signals (recent/
    /// pinned/frequent) attach in Phase 7 with their persistence layer.
    pub fn built_in_query(raw_query: &str) -> Self {
        let pages = built_in_page_definitions();
        let by_id: std::collections::HashMap<&str, &zettings_core::SettingDefinition> =
            pages.iter().map(|p| (p.id.as_str(), p)).collect();
        let hits: Vec<SearchHitDto> =
            rank_settings(raw_query, &pages, zettings_search::UsageSignals::default())
                .into_iter()
                .filter_map(|hit: SearchHit| {
                    let def = by_id.get(hit.setting_id.as_str())?;
                    Some(SearchHitDto {
                        setting_id: hit.setting_id,
                        title: def.title.clone(),
                        description: def.description.clone(),
                        category: def.category.as_str().to_owned(),
                        route: def.route.as_str().to_owned(),
                        score: hit.score,
                    })
                })
                .collect();
        Self {
            query: zettings_search::normalize_query(raw_query),
            hits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_ranks_display_page_for_screen_terms() {
        let res = SearchResponseDto::built_in_query("bluetooth");
        assert!(!res.hits.is_empty());
        assert_eq!(res.hits[0].setting_id, "devices.overview");
    }

    #[test]
    fn empty_query_yields_no_hits() {
        assert!(SearchResponseDto::built_in_query("   ").hits.is_empty());
    }
}
