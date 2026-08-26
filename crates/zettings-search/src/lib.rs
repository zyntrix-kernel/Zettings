//! Settings search index and ranking pipeline implementing the PLAN §5
//! baseline weights over the frozen registry snapshot.

mod index;
mod normalize;
mod usage;

pub use index::{MatchStage, QueryContext, SearchHit, SearchIndex};
pub use usage::UsageStats;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RankBoost {
    ExactTitle = 100,
    Keyword = 80,
    Alias = 70,
    PageOrCategory = 50,
    Description = 35,
    Semantic = 30,
    RecentlyUsed = 10,
    FuzzySpelling = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_weights_match_plan_baseline() {
        assert_eq!(RankBoost::ExactTitle as i32, 100);
        assert_eq!(RankBoost::Keyword as i32, 80);
        assert_eq!(RankBoost::Alias as i32, 70);
        assert_eq!(RankBoost::PageOrCategory as i32, 50);
        assert_eq!(RankBoost::Description as i32, 35);
        assert_eq!(RankBoost::Semantic as i32, 30);
        assert_eq!(RankBoost::RecentlyUsed as i32, 10);
        assert_eq!(RankBoost::FuzzySpelling as i32, 5);
    }
}
