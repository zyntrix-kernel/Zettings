//! Full-text + fuzzy search index for Zettings settings entries.
//!
//! Indexing pipeline:
//! 1. Each module, on mount, registers its settings entries (label, route,
//!    keywords, category) with [`Index::insert`].
//! 2. The index stores documents in `tantivy` with fields `label`,
//!    `keywords`, `route`, `module_id`, `category`.
//! 3. Queries are parsed: exact + tokenized + fuzzy (Levenshtein via
//!    `strsim`); ranked by recent + frequent + module weight.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// A settings entry registered by a module for indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsEntry {
    /// Globally unique entry id (`<module_id>:<slug>`).
    pub id: String,
    /// Owning module id.
    pub module_id: String,
    /// User-facing label, e.g. "Night Light".
    pub label: String,
    /// Broader category for grouping, e.g. "Display".
    pub category: String,
    /// Deep link, e.g. `/display/night-light`.
    pub route: String,
    /// Search keywords / synonyms.
    pub keywords: Vec<String>,
}

/// A ranked search hit returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// The matched settings entry.
    pub entry: SettingsEntry,
    /// Relevance score, higher is better.
    pub score: f32,
}

/// Errors surfaced by the search index.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// The query could not be parsed.
    #[error("query parse error: {0}")]
    Query(String),
    /// The index could not be opened.
    #[error("index open error: {0}")]
    Open(String),
}

/// In-memory index. Phase 4 replaces this with `tantivy` RAMDirectory-backed
/// index mounted under `~/.cache/zyntrix/zettings/search`.
#[derive(Default)]
pub struct Index {
    entries: std::sync::Arc<std::sync::RwLock<Vec<SettingsEntry>>>,
}

impl Index {
    /// Initialize a fresh empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an entry. Idempotent on `entry.id`.
    pub fn insert(&self, entry: SettingsEntry) {
        let mut guard = self.entries.write().expect("index lock poisoned");
        if let Some(existing) = guard.iter_mut().find(|e| e.id == entry.id) {
            *existing = entry;
        } else {
            guard.push(entry);
        }
    }

    /// Query the index.
    ///
    /// # Errors
    /// Currently never fails; matches the future `tantivy` API surface.
    pub fn search(&self, query: &str) -> Result<Vec<SearchHit>, SearchError> {
        let guard = self.entries.read().expect("index lock poisoned");
        let q = query.to_lowercase();
        let hits = guard
            .iter()
            .filter(|e| {
                e.label.to_lowercase().contains(&q)
                    || e.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .map(|e| SearchHit {
                entry: e.clone(),
                score: 1.0,
            })
            .collect();
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn night_light() -> SettingsEntry {
        SettingsEntry {
            id: "org.zyntrix.zettings.display:night-light".into(),
            module_id: "org.zyntrix.zettings.display".into(),
            label: "Night Light".into(),
            category: "Display".into(),
            route: "/display/night-light".into(),
            keywords: vec!["night light".into(), "warm".into(), "blue light".into()],
        }
    }

    #[test]
    fn search_finds_by_label_substring() {
        let idx = Index::new();
        idx.insert(night_light());
        let hits = idx.search("night").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.route, "/display/night-light");
    }

    #[test]
    fn search_finds_by_keyword() {
        let idx = Index::new();
        idx.insert(night_light());
        let hits = idx.search("blue").unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn search_returns_empty_for_unknown_term() {
        let idx = Index::new();
        idx.insert(night_light());
        assert!(idx.search("audio").unwrap().is_empty());
    }

    #[test]
    fn insert_is_idempotent_on_id() {
        let idx = Index::new();
        idx.insert(night_light());
        idx.insert(night_light());
        assert_eq!(idx.search("night").unwrap().len(), 1);
    }
}
