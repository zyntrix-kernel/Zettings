use std::collections::BTreeMap;
use std::sync::Arc;
use zettings_core::{BackendAdapter, BackendId};

/// Owns the registered adapter set and derives the reachability map consumed
/// by the capability layer (`EvalContext::reachable_backends`).
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn BackendAdapter>>,
}

impl AdapterRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn BackendAdapter>) {
        self.adapters.push(adapter);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<dyn BackendAdapter>> {
        self.adapters
            .iter()
            .find(|a| a.id().as_str() == id)
            .cloned()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }

    /// Probe every adapter concurrently; a down adapter degrades only itself.
    pub async fn reachability(&self) -> BTreeMap<String, bool> {
        let probes = self
            .adapters
            .iter()
            .map(|a| async move { (a.id(), a.health().await) });
        let results = probe_all(probes).await;
        results
            .into_iter()
            .map(|(id, health)| (id.to_string(), matches!(health, zettings_core::Health::Up)))
            .collect()
    }
}

pub(crate) async fn probe_all<I, F>(iter: I) -> Vec<(BackendId, zettings_core::Health)>
where
    I: IntoIterator<Item = F>,
    F: std::future::Future<Output = (BackendId, zettings_core::Health)>,
{
    use futures_util::future::join_all;
    join_all(iter).await
}
