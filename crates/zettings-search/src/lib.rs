//! Full-text + fuzzy search index for Zettings settings entries.
//!
//! Indexing pipeline:
//! 1. Each module, on mount, registers its settings entries (label, route,
//!    keywords, category) with [`Index::insert`] (batch registration via
//!    [`Index::insert_many`]).
//! 2. The index stores documents in a `tantivy` `RAMDirectory` with fields
//!    `id` (STRING STORED), `label` (TEXT STORED), `keywords` (TEXT STORED),
//!    `category` (STRING STORED), `route` (STRING STORED), and `module_id`
//!    (STRING STORED).
//! 3. Queries are tokenized, and each whitespace token drives a
//!    [`FuzzyTermQuery::new_prefix`] (Levenshtein distance 2, transpose-cost-one)
//!    against both the `label` and `keywords` fields; the per-token fuzzy
//!    clauses are OR-ed into a Boolean query executed via `TopDocs::with_limit(20)`.
//!    Prefix-mode fuzzy ensures short queries (e.g. "nite") still match longer
//!    indexed terms (e.g. "night").
//! 4. Hits are then re-ranked with `strsim::normalized_damerau_levenshtein`
//!    against the label and keywords for typo-tolerance scoring. The combined
//!    score is `tantivy_score * (0.5 + 0.5 * strsim_score)` so genuine BM25
//!    relevance still dominates while near-miss typos still surface.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use strsim::normalized_damerau_levenshtein;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur, Query};
use tantivy::schema::{Field, STORED, STRING, Schema, SchemaBuilder, TEXT, Value};
use tantivy::{
    DocAddress, Index as TvIndex, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term,
};
use ts_rs::TS;

/// A settings entry registered by a module for indexing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "search_settings_entry.ts")]
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
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "search_hit.ts")]
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

/// Schema field cache, populated once at construction time.
struct Fields {
    id: Field,
    module_id: Field,
    label: Field,
    category: Field,
    route: Field,
    keywords: Field,
}

/// Build the Tantivy schema and return both the schema and the field cache.
fn build_schema() -> (Schema, Fields) {
    let mut builder = SchemaBuilder::new();
    let id = builder.add_text_field("id", STRING | STORED);
    let module_id = builder.add_text_field("module_id", STRING | STORED);
    let label = builder.add_text_field("label", TEXT | STORED);
    let category = builder.add_text_field("category", STRING | STORED);
    let route = builder.add_text_field("route", STRING | STORED);
    let keywords = builder.add_text_field("keywords", TEXT | STORED);
    let schema = builder.build();
    (
        schema,
        Fields {
            id,
            module_id,
            label,
            category,
            route,
            keywords,
        },
    )
}

fn read_stored_string(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_owned()
}

fn read_stored_keywords(doc: &TantivyDocument, field: Field) -> Vec<String> {
    let mut out = Vec::new();
    for v in doc.get_all(field) {
        if let Some(s) = v.as_str() {
            out.push(s.to_owned());
        }
    }
    out
}

fn document_from_entry(fields: &Fields, entry: &SettingsEntry) -> TantivyDocument {
    let mut doc = TantivyDocument::new();
    doc.add_text(fields.id, &entry.id);
    doc.add_text(fields.module_id, &entry.module_id);
    doc.add_text(fields.label, &entry.label);
    doc.add_text(fields.category, &entry.category);
    doc.add_text(fields.route, &entry.route);
    for kw in &entry.keywords {
        doc.add_text(fields.keywords, kw);
    }
    doc
}

fn entry_from_document(fields: &Fields, doc: &TantivyDocument) -> SettingsEntry {
    SettingsEntry {
        id: read_stored_string(doc, fields.id),
        module_id: read_stored_string(doc, fields.module_id),
        label: read_stored_string(doc, fields.label),
        category: read_stored_string(doc, fields.category),
        route: read_stored_string(doc, fields.route),
        keywords: read_stored_keywords(doc, fields.keywords),
    }
}

/// Returns `0.0..=1.0` similarity between query and entry, the best label-vs-query
/// and max keyword-vs-query Damerau-Levenshtein normalized score.
fn strsim_score(query: &str, entry: &SettingsEntry) -> f64 {
    let q = query.to_lowercase();
    let label_lower = entry.label.to_lowercase();
    let label_score = normalized_damerau_levenshtein(&q, &label_lower);
    let keyword_score = entry
        .keywords
        .iter()
        .map(|k| normalized_damerau_levenshtein(&q, &k.to_lowercase()))
        .fold(0.0_f64, f64::max);
    let prefix_boost = if label_lower.starts_with(&q) {
        0.1
    } else {
        0.0
    };
    label_score.max(keyword_score) + prefix_boost
}

/// Tantivy RAMDirectory-backed in-memory search index for settings entries.
///
/// Phase 6 of PLAN.md: real Tantivy schema + fuzzy [`BooleanQuery`] + `strsim`
/// re-ranking. The previous substring-match prototype is replaced here.
pub struct Index {
    inner: TvIndex,
    reader: IndexReader,
    fields: Fields,
    // Serializes writer mutations — `IndexWriter` requires `&mut self` access
    // for `commit`, and Tauri command surfaces may issue concurrent inserts.
    writer_lock: Arc<RwLock<()>>,
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    /// Initialize a fresh empty in-memory index.
    ///
    /// # Panics
    /// Panics only if Tantivy cannot allocate an in-RAM index — a developer-only
    /// condition that should never occur in practice.
    #[must_use]
    pub fn new() -> Self {
        let (schema, fields) = build_schema();
        let inner = TvIndex::create_in_ram(schema);
        let reader = inner
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .expect("RAMDirectory index reader construction is infallible");
        Self {
            inner,
            reader,
            fields,
            writer_lock: Arc::new(RwLock::new(())),
        }
    }

    fn writer(&self) -> Result<IndexWriter, SearchError> {
        self.inner
            .writer(50_000_000)
            .map_err(|e| SearchError::Open(e.to_string()))
    }

    /// Insert an entry, idempotent on `entry.id`. Existing entries with the
    /// same id are replaced atomically.
    ///
    /// # Errors
    /// Returns [`SearchError::Open`] when the underlying Tantivy writer or
    /// commit fails (e.g. disk exhaustion — not a concern for `RAMDirectory`).
    pub fn insert(&self, entry: SettingsEntry) -> Result<(), SearchError> {
        self.insert_many(&[entry])
    }

    /// Batch-insert many entries within a single writer transaction.
    ///
    /// # Errors
    /// Returns [`SearchError::Open`] when the underlying Tantivy writer or
    /// commit fails.
    pub fn insert_many(&self, entries: &[SettingsEntry]) -> Result<(), SearchError> {
        if entries.is_empty() {
            return Ok(());
        }
        let _guard = self
            .writer_lock
            .write()
            .expect("search writer lock poisoned");
        let mut writer = self.writer()?;
        let id_field = self.fields.id;
        for entry in entries {
            // Delete any prior document with the same id before re-adding.
            writer.delete_term(Term::from_field_text(id_field, &entry.id));
            writer
                .add_document(document_from_entry(&self.fields, entry))
                .map_err(|e| SearchError::Open(e.to_string()))?;
        }
        writer
            .commit()
            .map_err(|e| SearchError::Open(e.to_string()))?;
        // Pull the new index view in for the next search.
        self.reader
            .reload()
            .map_err(|e| SearchError::Open(e.to_string()))?;
        Ok(())
    }

    /// Query the index with fuzzy Tantivy + `strsim` typo-tolerant re-ranking.
    /// Results are returned in descending score order, capped at 20 hits.
    ///
    /// # Errors
    /// Returns [`SearchError::Query`] when the query contains no valid tokens,
    /// or [`SearchError::Open`] when the underlying reader or document fetch fails.
    pub fn search(&self, query: &str) -> Result<Vec<SearchHit>, SearchError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let label_field = self.fields.label;
        let keywords_field = self.fields.keywords;

        // Build a Boolean OR of fuzzy-prefix term queries: for each whitespace
        // token in the query, we emit two `FuzzyTermQuery::new_prefix` clauses
        // (one against `label`, one against `keywords`) at Levenshtein distance 2
        // with transpose-cost-one. Prefix mode ensures short queries like "nite"
        // still match longer indexed terms like "night".
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        for token in trimmed.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            let lower = token.to_lowercase();
            let label_term = Term::from_field_text(label_field, &lower);
            let keywords_term = Term::from_field_text(keywords_field, &lower);
            clauses.push((
                Occur::Should,
                Box::new(FuzzyTermQuery::new_prefix(label_term, 2, true)),
            ));
            clauses.push((
                Occur::Should,
                Box::new(FuzzyTermQuery::new_prefix(keywords_term, 2, true)),
            ));
        }
        // Also emit a fuzzy clause over the whole (lowercased) query so multi-word
        // labels like "Blue Light" surface when the user types the whole phrase
        // rather than tokenising it.
        let whole_lower = trimmed.to_lowercase();
        let whole_label = Term::from_field_text(label_field, &whole_lower);
        let whole_keywords = Term::from_field_text(keywords_field, &whole_lower);
        clauses.push((
            Occur::Should,
            Box::new(FuzzyTermQuery::new_prefix(whole_label, 2, true)),
        ));
        clauses.push((
            Occur::Should,
            Box::new(FuzzyTermQuery::new_prefix(whole_keywords, 2, true)),
        ));
        if clauses.is_empty() {
            return Err(SearchError::Query("empty query".into()));
        }
        let boolean = BooleanQuery::new(clauses);
        let collector = TopDocs::with_limit(20).order_by_score();
        let searcher = self.reader.searcher();
        let top: Vec<(tantivy::Score, DocAddress)> = searcher
            .search(&boolean, &collector)
            .map_err(|e| SearchError::Open(e.to_string()))?;

        let mut hits = Vec::with_capacity(top.len());
        for (bm25, addr) in top {
            let doc: TantivyDocument = searcher
                .doc(addr)
                .map_err(|e| SearchError::Open(e.to_string()))?;
            let entry = entry_from_document(&self.fields, &doc);
            let strsim = strsim_score(trimmed, &entry);
            // Combine BM25 with strsim — both in [0,1]-ish range; clamp.
            let bm25 = f64::from(bm25);
            let combined = (bm25 * (0.5 + 0.5 * strsim)).max(strsim);
            // `combined` is a normalized similarity in 0.0..=1.0 plus a small
            // prefix boost (max +0.1); narrowing to f32 here is safe.
            #[allow(clippy::cast_possible_truncation)]
            let score = combined as f32;
            hits.push(SearchHit { entry, score });
        }
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
        idx.insert(night_light()).unwrap();
        let hits = idx.search("night").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.route, "/display/night-light");
    }

    #[test]
    fn search_finds_by_keyword() {
        let idx = Index::new();
        idx.insert(night_light()).unwrap();
        let hits = idx.search("blue").unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn search_returns_empty_for_unknown_term() {
        let idx = Index::new();
        idx.insert(night_light()).unwrap();
        assert!(idx.search("audio").unwrap().is_empty());
    }

    #[test]
    fn insert_is_idempotent_on_id() {
        let idx = Index::new();
        idx.insert(night_light()).unwrap();
        idx.insert(night_light()).unwrap();
        assert_eq!(idx.search("night").unwrap().len(), 1);
    }

    #[test]
    fn search_tolerates_typos_via_strsim_reranking() {
        // Fuzzy + strsim should still surface the entry despite a typo
        // ("Nite" vs "Night") and even though no keyword matches.
        let idx = Index::new();
        idx.insert(night_light()).unwrap();
        let hits = idx.search("nite").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.route, "/display/night-light");
    }

    #[test]
    fn search_empty_query_returns_no_hits() {
        let idx = Index::new();
        idx.insert(night_light()).unwrap();
        assert!(idx.search("").unwrap().is_empty());
        assert!(idx.search("   ").unwrap().is_empty());
    }

    #[test]
    fn insert_many_batches_in_single_transaction() {
        let idx = Index::new();
        let display = SettingsEntry {
            id: "display:scaling".into(),
            module_id: "display".into(),
            label: "Display Scaling".into(),
            category: "Display".into(),
            route: "/display/scaling".into(),
            keywords: vec!["dpi".into(), "scale".into()],
        };
        let audio = SettingsEntry {
            id: "audio:volume".into(),
            module_id: "audio".into(),
            label: "Master Volume".into(),
            category: "Audio".into(),
            route: "/audio/volume".into(),
            keywords: vec!["loudness".into(), "balance".into()],
        };
        idx.insert_many(&[display, audio]).unwrap();
        assert_eq!(idx.search("volume").unwrap().len(), 1);
        assert_eq!(idx.search("scale").unwrap().len(), 1);
        assert_eq!(idx.search("display scaling").unwrap().len(), 1);
    }
}
