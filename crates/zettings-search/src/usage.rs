use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageStats {
    #[serde(default)]
    pub recent: Vec<String>,
    #[serde(default)]
    pub frequent: HashMap<String, u32>,
    #[serde(default)]
    pub pinned: Vec<String>,
}

impl UsageStats {
    #[must_use]
    pub fn is_recent(&self, setting_id: &str) -> bool {
        self.recent.iter().any(|id| id == setting_id)
    }

    #[must_use]
    pub fn frequency(&self, setting_id: &str) -> u32 {
        self.frequent.get(setting_id).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn is_pinned(&self, setting_id: &str) -> bool {
        self.pinned.iter().any(|id| id == setting_id)
    }

    pub fn record_use(&mut self, setting_id: &str) {
        if self.recent.first().map(String::as_str) != Some(setting_id) {
            self.recent.retain(|id| id != setting_id);
            self.recent.insert(0, String::from(setting_id));
            self.recent.truncate(20);
        }
        *self.frequent.entry(String::from(setting_id)).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_recency_and_frequency() {
        let mut stats = UsageStats::default();
        stats.record_use("a");
        stats.record_use("b");
        stats.record_use("a");
        assert_eq!(stats.recent, vec![String::from("a"), String::from("b")]);
        assert_eq!(stats.frequency("a"), 2);
        assert!(stats.is_recent("a"));
        assert!(!stats.is_pinned("a"));
    }

    #[test]
    fn recent_is_capped() {
        let mut stats = UsageStats::default();
        for i in 0..30 {
            stats.record_use(&format!("s{i}"));
        }
        assert_eq!(stats.recent.len(), 20);
        assert!(!stats.is_recent("s0"));
        assert!(stats.is_recent("s29"));
    }
}
