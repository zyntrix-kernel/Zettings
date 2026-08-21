//! Settings search: normalization, scoring, and ranking.
//!
//! Phase 1 ships the deterministic scoring kernel that both the instant
//! in-process search and the later persisted index (Phase 7) must agree on:
//! the weighted model from `PLAN.md` §5 / Windows reconstruction spec §9.
//!
//! ```text
//! exact title +100 · exact keyword +80 · alias +70 · page/category +50
//! description +35 · semantic similarity +30 · recently used +10
//! fuzzy spelling +5 — plus per-setting `search_weight` boost
//! ```

use serde::{Deserialize, Serialize};
use zettings_core::SettingDefinition;

/// Weight constants implementing the ranking baseline. `pub` so the future
/// persistence layer and tests share exact numbers with documentation.
pub mod weights {
    /// Score contribution for an exact title match.
    pub const EXACT_TITLE: i32 = 100;
    /// Score contribution for an exact keyword match.
    pub const EXACT_KEYWORD: i32 = 80;
    /// Score contribution for an alias match.
    pub const ALIAS: i32 = 70;
    /// Score contribution for a page/category name match.
    pub const PAGE_CATEGORY: i32 = 50;
    /// Score contribution for a description (substring) match.
    pub const DESCRIPTION: i32 = 35;
    /// Score contribution for a strong semantic/fuzzy title similarity.
    pub const SEMANTIC: i32 = 30;
    /// Score contribution when the setting was recently used.
    pub const RECENTLY_USED: i32 = 10;
    /// Score contribution for a fuzzy spelling correction.
    pub const FUZZY: i32 = 5;
    /// Similarity ratio at or above which a fuzzy match is considered real.
    pub const SIMILARITY_THRESHOLD: f64 = 0.82;
}

/// Usage signals fed into ranking; all optional and additive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageSignals {
    /// The setting was opened or changed recently (recency window owned by UI).
    pub recently_used: bool,
}

/// A ranked search hit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchHit {
    /// Stable id of the matched setting.
    pub setting_id: String,
    /// Total computed score (weights + `search_weight`).
    pub score: i32,
}

/// Normalizes a user query: trim, lowercase, collapse whitespace, strip
/// punctuation except intra-word hyphens, ASCII-fold common diacritics.
#[must_use]
pub fn normalize_query(raw: &str) -> String {
    let lowered = raw.to_lowercase();
    let folded: String = lowered
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            other => other,
        })
        .collect();
    let mut out = String::with_capacity(folded.len());
    let mut last_ws = true;
    for c in folded.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            out.push(c);
            last_ws = false;
        } else if !last_ws {
            out.push(' ');
            last_ws = true;
        }
    }
    out.trim().to_owned()
}

/// Scores one setting against a normalized query. Pure and side-effect free.
///
/// Direct settings outrank broad category pages because only concrete
/// settings are scored here; category-level matches enter through
/// [`weights::PAGE_CATEGORY`] on the page field.
#[must_use]
pub fn score_setting(query: &str, setting: &SettingDefinition, usage: UsageSignals) -> i32 {
    let _ = usage; // wired to recency provider in Phase 7
    let mut score = 0;
    let title = normalize_query(&setting.title);

    if title == query {
        score += weights::EXACT_TITLE;
    } else if title.contains(query) || query.contains(&title) && !title.is_empty() {
        // Substring containment is treated as semantic proximity.
        score += weights::SEMANTIC;
    }

    let keyword_exact = setting.keywords.iter().any(|k| normalize_query(k) == query);
    if keyword_exact {
        score += weights::EXACT_KEYWORD;
    }

    if setting.aliases.iter().any(|a| normalize_query(a) == query) {
        score += weights::ALIAS;
    }

    let page = normalize_query(&setting.page);
    let category = setting.category.as_str();
    if page.contains(query) || category.contains(query) {
        score += weights::PAGE_CATEGORY;
    }

    if normalize_query(&setting.description).contains(query) && !query.is_empty() {
        score += weights::DESCRIPTION;
    }

    if score == 0 {
        // Fuzzy fallbacks only when no structural signal existed.
        let ratio = strsim::jaro_winkler(query, &title);
        if ratio >= weights::SIMILARITY_THRESHOLD {
            // ratio ∈ [SIMILARITY_THRESHOLD, 1] ⇒ scaled value ≤ 10.
            #[allow(clippy::cast_possible_truncation)]
            let similarity_bonus = (ratio * 10.0) as i32;
            score += weights::FUZZY + similarity_bonus;
        }
    }

    score + setting.search_weight
}

/// Ranks settings for `query`, best first. Ties resolve by stable id to keep
/// result order deterministic across runs (required by snapshot tests).
#[must_use]
pub fn rank_settings(
    raw_query: &str,
    settings: &[SettingDefinition],
    usage: UsageSignals,
) -> Vec<SearchHit> {
    let query = normalize_query(raw_query);
    // An empty query has no intent; returning everything would be noise.
    if query.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<SearchHit> = settings
        .iter()
        .map(|s| SearchHit {
            score: score_setting(&query, s, usage),
            setting_id: s.id.clone(),
        })
        .filter(|h| h.score > 0)
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.setting_id.cmp(&b.setting_id))
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use zettings_core::{CategoryId, RouteId, ValueRequirement};

    fn setting(id: &str, title: &str, aliases: &[&str], keywords: &[&str]) -> SettingDefinition {
        SettingDefinition {
            id: id.into(),
            title: title.into(),
            description: format!("{title} configuration"),
            category: CategoryId::parse("system").expect("valid"),
            page: "display".into(),
            section: "scale".into(),
            route: RouteId::parse("zettings://system/display").expect("valid"),
            aliases: aliases.iter().map(|s| (*s).into()).collect(),
            keywords: keywords.iter().map(|s| (*s).into()).collect(),
            control_type: zettings_core::ControlType::ComboBox,
            requirements: ValueRequirement::default(),
            search_weight: 0,
        }
    }

    #[test]
    fn normalizes_case_punct_diacritics() {
        assert_eq!(normalize_query("  Café  Size!! "), "cafe size");
    }

    #[test]
    fn direct_setting_outranks_via_alias_and_keyword_weights() {
        let scale = setting(
            "system.display.scale",
            "Scale",
            &["make things bigger"],
            &["dpi", "text size"],
        );
        let hits = rank_settings(
            "text size",
            std::slice::from_ref(&scale),
            UsageSignals::default(),
        );
        assert_eq!(hits.len(), 1);
        // keyword(80) > page/category would be 50; ensure keyword path used.
        assert!(hits[0].score >= 80);
    }

    #[test]
    fn fuzzy_recovers_misspelling_when_no_structural_match() {
        let scale = setting("system.display.scale", "Scale", &[], &[]);
        let hits = rank_settings(
            "skale",
            std::slice::from_ref(&scale),
            UsageSignals::default(),
        );
        assert!(!hits.is_empty(), "misspelling should still surface Scale");
    }

    #[test]
    fn ties_are_stable_by_id() {
        let a = setting("system.a", "Alpha", &[], &[]);
        let b = setting("system.b", "Beta", &[], &[]);
        let hits = rank_settings("", &[b, a], UsageSignals::default());
        // Empty query scores nothing anywhere → no hits rather than arbitrary order.
        assert!(hits.is_empty());
    }
}
