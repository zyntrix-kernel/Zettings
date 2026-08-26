use std::sync::Arc;
use zettings_backends::{AdapterRegistry, MockAdapter};
use zettings_core::BackendId;

#[tokio::test]
async fn registry_aggregates_reachability_per_adapter() {
    let mut registry = AdapterRegistry::new();
    registry.register(Arc::new(
        MockAdapter::new(BackendId::timedate())
            .with_value("ntp", zettings_core::SettingValue::Bool(true)),
    ));
    registry.register(Arc::new(MockAdapter::new(BackendId::bluetooth()).down()));
    assert_eq!(registry.len(), 2);

    let reach = registry.reachability().await;
    assert_eq!(reach.get("timedate"), Some(&true));
    assert_eq!(reach.get("bluetooth"), Some(&false));

    let timedate = registry.get("timedate").unwrap();
    assert_eq!(timedate.id().as_str(), "timedate");
    assert!(registry.get("audio").is_none());
}

#[tokio::test]
async fn empty_registry_reports_no_backends() {
    let registry = AdapterRegistry::new();
    assert!(registry.is_empty());
    assert!(registry.reachability().await.is_empty());
}
