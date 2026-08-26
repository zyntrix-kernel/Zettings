use zettings_core::{
    BackendId, CapabilityRequirement, Category, CategoryId, ControlKind, I18nKey, IconToken, Page,
    PageId, PageTemplate, ReadBinding, RegistryBuilder, RegistrySnapshot, Section,
    SettingDefinition, SettingId, SettingMetadata,
};

use zettings_search::{MatchStage, QueryContext, SearchIndex, UsageStats};

fn setting(
    id: &str,
    page: &str,
    title: &str,
    desc: Option<&str>,
    aliases: &[&str],
    keywords: &[&str],
) -> SettingDefinition {
    SettingDefinition {
        id: SettingId(String::from(id)),
        metadata: SettingMetadata {
            title: I18nKey(String::from(title)),
            description: desc.map(|d| I18nKey(String::from(d))),
            aliases: aliases.iter().map(|a| String::from(*a)).collect(),
            keywords: keywords.iter().map(|k| String::from(*k)).collect(),
            search_weight: 0,
        },
        page_id: PageId(String::from(page)),
        section_id: String::from("main"),
        control: ControlKind::Toggle,
        read: ReadBinding {
            backend: BackendId::timedate(),
            key: zettings_core::ValueKey(String::from("k")),
            poll: zettings_core::PollPolicy::OnDemand,
        },
        write: None,
        permission: CapabilityRequirement::SessionOnly,
        hardware_dependency: None,
        reboot_hint: zettings_core::RebootHint::None,
        default_value: None,
    }
}

fn snapshot_with(entries: Vec<SettingDefinition>, category: &str, page: &str) -> RegistrySnapshot {
    let mut builder = RegistryBuilder::new(&[BackendId::timedate()]);
    builder
        .register_category(Category {
            id: CategoryId(String::from(category)),
            title: I18nKey(String::from("cat")),
            icon: IconToken(String::from("icon")),
            order: 1,
            keywords: vec![],
            availability: None,
            pages: vec![Page {
                id: PageId(String::from(page)),
                category_id: CategoryId(String::from(category)),
                title: I18nKey(String::from("page")),
                description: None,
                template: PageTemplate::SettingsPage,
                sections: vec![Section {
                    id: String::from("main"),
                    title: None,
                    settings: entries,
                    visibility: None,
                }],
                required_capabilities: vec![],
            }],
        })
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn exact_title_outranks_alias_and_description() {
    let snapshot = snapshot_with(
        vec![
            setting("a.ntp", "datetime", "settings.timedate.ntp", None, &[], &[]),
            setting(
                "a.wifi",
                "datetime",
                "Wireless",
                Some("network time sync"),
                &[],
                &[],
            ),
        ],
        "system",
        "datetime",
    );
    let index = SearchIndex::build(&snapshot, &UsageStats::default());
    let hits = index.query("ntp", &QueryContext::default(), &UsageStats::default());
    assert_eq!(hits[0].setting_id.0, "a.ntp");
    assert!(hits[0].stages.contains(&MatchStage::ExactTitle));
    assert_eq!(hits[0].score, 100);
}

#[test]
fn matching_is_diacritic_and_case_insensitive() {
    let snapshot = snapshot_with(
        vec![setting(
            "a.night",
            "display",
            "Nattljus natt-läge",
            None,
            &[],
            &[],
        )],
        "system",
        "display",
    );
    let index = SearchIndex::build(&snapshot, &UsageStats::default());
    let hits = index.query("NATT", &QueryContext::default(), &UsageStats::default());
    assert!(!hits.is_empty());
}

#[test]
fn fuzzy_requires_long_token_and_small_distance() {
    let snapshot = snapshot_with(
        vec![setting(
            "a.bt",
            "bluetooth",
            "Bluetooth devices",
            None,
            &["bluetooth"],
            &[],
        )],
        "devices",
        "bluetooth",
    );
    let index = SearchIndex::build(&snapshot, &UsageStats::default());
    let hits = index.query("blutooh", &QueryContext::default(), &UsageStats::default());
    assert!(
        hits.iter()
            .any(|h| h.stages.contains(&MatchStage::FuzzySpelling))
    );
}

#[test]
fn recent_usage_boosts_score() {
    let snapshot = snapshot_with(
        vec![
            setting("a.one", "datetime", "alpha one", None, &[], &[]),
            setting("a.two", "datetime", "alpha two", None, &[], &[]),
        ],
        "system",
        "datetime",
    );
    let mut usage = UsageStats::default();
    usage.record_use("a.two");
    let index = SearchIndex::build(&snapshot, &usage);
    let hits = index.query("alpha", &QueryContext::default(), &usage);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].setting_id.0, "a.two");
    assert!(hits[0].score > hits[1].score);
}

#[test]
fn empty_query_returns_nothing_and_ties_follow_registry_order() {
    let snapshot = snapshot_with(
        vec![
            setting("a.first", "datetime", "gamma", None, &[], &[]),
            setting("a.second", "datetime", "gamma twin", None, &[], &[]),
        ],
        "system",
        "datetime",
    );
    let index = SearchIndex::build(&snapshot, &UsageStats::default());
    assert!(
        index
            .query("   ", &QueryContext::default(), &UsageStats::default())
            .is_empty()
    );

    let hits = index.query("gamma", &QueryContext::default(), &UsageStats::default());
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].setting_id.0, "a.first");
    assert_eq!(hits[1].setting_id.0, "a.second");
}
