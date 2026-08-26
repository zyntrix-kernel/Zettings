use zettings_core::{RegistrySnapshot, SettingId};

use crate::RankBoost;
use crate::normalize::{edit_distance_within, fold, tokenize};
use crate::usage::UsageStats;

const FUZZY_MAX_EDITS: usize = 2;
const FUZZY_MIN_TOKEN_LEN: usize = 4;
const MAX_HITS: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchStage {
    ExactTitle,
    Keyword,
    Alias,
    PageOrCategory,
    Description,
    FuzzySpelling,
}

impl MatchStage {
    #[must_use]
    pub fn boost(self) -> i32 {
        match self {
            Self::ExactTitle => RankBoost::ExactTitle as i32,
            Self::Keyword => RankBoost::Keyword as i32,
            Self::Alias => RankBoost::Alias as i32,
            Self::PageOrCategory => RankBoost::PageOrCategory as i32,
            Self::Description => RankBoost::Description as i32,
            Self::FuzzySpelling => RankBoost::FuzzySpelling as i32,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SearchHit {
    pub setting_id: SettingId,
    pub route_canonical: String,
    pub score: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<MatchStage>,
}

#[derive(Clone, Debug, Default)]
pub struct QueryContext {
    pub current_category: Option<String>,
}

struct IndexedSetting {
    setting_id: SettingId,
    route_canonical: String,
    category_folded: String,
    page_folded: String,
    title_folded: String,
    description_folded: String,
    alias_tokens: Vec<String>,
    keyword_tokens: Vec<String>,
    title_tokens: Vec<String>,
    description_tokens: Vec<String>,
    search_weight: i32,
}

#[derive(Default)]
pub struct SearchIndex {
    entries: Vec<IndexedSetting>,
}

impl SearchIndex {
    #[must_use]
    pub fn build(snapshot: &RegistrySnapshot, _usage: &UsageStats) -> Self {
        let mut entries = Vec::new();
        for category in &snapshot.categories {
            let category_folded = fold(&category.id.0);
            for page in &category.pages {
                let route = page.route();
                let page_folded = fold(&page.id.0);
                for section in &page.sections {
                    for setting in &section.settings {
                        let title_folded = fold(&setting.metadata.title.0);
                        let description_folded = setting
                            .metadata
                            .description
                            .as_ref()
                            .map(|d| fold(&d.0))
                            .unwrap_or_default();
                        entries.push(IndexedSetting {
                            setting_id: setting.id.clone(),
                            route_canonical: route.canonical(),
                            category_folded: category_folded.clone(),
                            page_folded: page_folded.clone(),
                            alias_tokens: setting
                                .metadata
                                .aliases
                                .iter()
                                .flat_map(|a| tokenize(&fold(a)))
                                .collect(),
                            keyword_tokens: setting
                                .metadata
                                .keywords
                                .iter()
                                .map(|k| fold(k))
                                .collect(),
                            title_tokens: tokenize(&title_folded),
                            description_tokens: tokenize(&description_folded),
                            title_folded,
                            description_folded,
                            search_weight: setting.metadata.search_weight,
                        });
                    }
                }
            }
        }
        Self { entries }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn query(&self, raw: &str, ctx: &QueryContext, usage: &UsageStats) -> Vec<SearchHit> {
        let query_folded = fold(raw.trim());
        if query_folded.is_empty() {
            return Vec::new();
        }
        let query_tokens = tokenize(&query_folded);

        let mut matches: Vec<(usize, i32, Vec<MatchStage>)> = Vec::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            let mut score = 0;
            let mut stages: Vec<MatchStage> = Vec::new();

            if entry.title_folded.contains(&query_folded) {
                score += MatchStage::ExactTitle.boost();
                stages.push(MatchStage::ExactTitle);
            } else {
                if query_tokens.iter().any(|qt| {
                    entry
                        .keyword_tokens
                        .iter()
                        .any(|kt| kt == qt || kt.contains(qt))
                }) {
                    score += MatchStage::Keyword.boost();
                    stages.push(MatchStage::Keyword);
                }
                if query_tokens
                    .iter()
                    .any(|qt| entry.alias_tokens.iter().any(|at| at.contains(qt)))
                {
                    score += MatchStage::Alias.boost();
                    stages.push(MatchStage::Alias);
                }
                if entry.page_folded.contains(&query_folded)
                    || entry.category_folded.contains(&query_folded)
                {
                    score += MatchStage::PageOrCategory.boost();
                    stages.push(MatchStage::PageOrCategory);
                }
                if !entry.description_folded.is_empty()
                    && (entry.description_folded.contains(&query_folded)
                        || query_tokens
                            .iter()
                            .any(|qt| entry.description_tokens.iter().any(|dt| dt == qt)))
                {
                    score += MatchStage::Description.boost();
                    stages.push(MatchStage::Description);
                }
                if score == 0
                    && query_tokens.len() == 1
                    && query_tokens[0].len() >= FUZZY_MIN_TOKEN_LEN
                    && (entry
                        .title_tokens
                        .iter()
                        .any(|tt| edit_distance_within(tt, &query_tokens[0], FUZZY_MAX_EDITS))
                        || entry
                            .alias_tokens
                            .iter()
                            .any(|at| edit_distance_within(at, &query_tokens[0], FUZZY_MAX_EDITS)))
                {
                    score += MatchStage::FuzzySpelling.boost();
                    stages.push(MatchStage::FuzzySpelling);
                }
            }

            if score > 0 {
                if ctx
                    .current_category
                    .as_deref()
                    .is_some_and(|c| fold(c) == entry.category_folded)
                {
                    score += MatchStage::PageOrCategory.boost();
                }
                if usage.is_recent(entry.setting_id.0.as_str()) {
                    score += RankBoost::RecentlyUsed as i32;
                }
                if usage.is_pinned(entry.setting_id.0.as_str()) {
                    score += MatchStage::Keyword.boost() / 2;
                }
                score += entry.search_weight;
                matches.push((idx, score, stages));
            }
        }

        matches.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        matches
            .into_iter()
            .take(MAX_HITS)
            .map(|(idx, score, stages)| {
                let e = &self.entries[idx];
                SearchHit {
                    setting_id: e.setting_id.clone(),
                    route_canonical: e.route_canonical.clone(),
                    score,
                    stages,
                }
            })
            .collect()
    }
}
