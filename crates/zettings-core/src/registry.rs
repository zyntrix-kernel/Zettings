use crate::backend::{ReadBinding, WriteAction};
use crate::capability::{CapabilityRequirement, HardwareProbe};
use crate::route::{CategoryId, PageId};
use crate::{BackendId, RebootHint, SettingValue};

pub const REGISTRY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SettingId(pub String);

impl std::fmt::Display for SettingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct I18nKey(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct IconToken(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlKind {
    Toggle,
    Slider,
    ComboBox,
    TextField,
    Button,
    ActionRow,
    Expander,
    ColorPicker,
    RadioGroup,
    FilePicker,
    DateTimePicker,
    InfoBadge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageTemplate {
    CategoryHub,
    SettingsPage,
    DeviceList,
    Detail,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SettingMetadata {
    pub title: I18nKey,
    pub description: Option<I18nKey>,
    pub aliases: Vec<String>,
    pub keywords: Vec<String>,
    pub search_weight: i32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SettingDefinition {
    pub id: SettingId,
    pub metadata: SettingMetadata,
    pub page_id: PageId,
    pub section_id: String,
    pub control: ControlKind,
    pub read: ReadBinding,
    pub write: Option<WriteAction>,
    pub permission: CapabilityRequirement,
    pub hardware_dependency: Option<HardwareProbe>,
    pub reboot_hint: RebootHint,
    pub default_value: Option<SettingValue>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub id: String,
    pub title: Option<I18nKey>,
    pub settings: Vec<SettingDefinition>,
    pub visibility: Option<CapabilityRequirement>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub id: PageId,
    pub category_id: CategoryId,
    pub title: I18nKey,
    pub description: Option<I18nKey>,
    pub template: PageTemplate,
    pub sections: Vec<Section>,
    pub required_capabilities: Vec<CapabilityRequirement>,
}

impl Page {
    #[must_use]
    pub fn route(&self) -> crate::Route {
        let mut route = crate::Route::new(&self.category_id.0);
        route.page = Some(PageId(String::from(&self.id.0)));
        route
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: CategoryId,
    pub title: I18nKey,
    pub icon: IconToken,
    pub order: i32,
    pub keywords: Vec<String>,
    pub availability: Option<CapabilityRequirement>,
    pub pages: Vec<Page>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegistrySnapshot {
    pub schema_version: u32,
    pub categories: Vec<Category>,
}

impl RegistrySnapshot {
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.categories.iter().map(|c| c.pages.len()).sum()
    }

    #[must_use]
    pub fn setting_count(&self) -> usize {
        self.categories
            .iter()
            .flat_map(|c| c.pages.iter())
            .flat_map(|p| p.sections.iter())
            .map(|s| s.settings.len())
            .sum()
    }

    #[must_use]
    pub fn find_page(&self, id: &PageId) -> Option<&Page> {
        self.categories
            .iter()
            .flat_map(|c| c.pages.iter())
            .find(|p| &p.id == id)
    }
}

#[derive(Debug, Default)]
pub struct RegistryBuilder {
    known_backends: Vec<BackendId>,
    categories: Vec<Category>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("duplicate category id {0}")]
    DuplicateCategory(String),
    #[error("duplicate page id {0}")]
    DuplicatePage(String),
    #[error("duplicate setting id {0}")]
    DuplicateSetting(String),
    #[error("setting {setting} references unknown backend {backend}")]
    UnknownBackend { setting: String, backend: String },
    #[error("page {0} has no sections; every page needs at least one")]
    EmptyPage(String),
    #[error("section {section} on page {page} has no settings")]
    EmptySection { page: String, section: String },
    #[error("category {0} has no pages")]
    EmptyCategory(String),
}

impl RegistryBuilder {
    #[must_use]
    pub fn new(known_backends: &[BackendId]) -> Self {
        Self {
            known_backends: known_backends.to_vec(),
            categories: Vec::new(),
        }
    }

    /// # Errors
    ///
    /// Returns [`RegistryError`] when the category id collides with an
    /// already-registered category.
    pub fn register_category(&mut self, category: Category) -> Result<(), RegistryError> {
        if self.categories.iter().any(|c| c.id == category.id) {
            return Err(RegistryError::DuplicateCategory(category.id.0));
        }
        self.categories.push(category);
        Ok(())
    }
    /// # Errors
    ///
    /// Returns [`RegistryError`] when registry invariants are violated:
    /// duplicate ids, empty categories/pages/sections, routes that do not
    /// resolve to their own page definition, or bindings naming unregistered
    /// backends.
    pub fn build(self) -> Result<RegistrySnapshot, RegistryError> {
        let mut seen_pages = std::collections::HashSet::new();
        let mut seen_settings = std::collections::HashSet::new();

        for category in &self.categories {
            if category.pages.is_empty() {
                return Err(RegistryError::EmptyCategory(category.id.0.clone()));
            }
            for page in &category.pages {
                if !seen_pages.insert(page.id.clone()) {
                    return Err(RegistryError::DuplicatePage(page.id.0.clone()));
                }
                if page.category_id != category.id {
                    return Err(RegistryError::DuplicatePage(format!(
                        "{} (mismatched category {})",
                        page.id.0, category.id.0
                    )));
                }
                if page.sections.is_empty() {
                    return Err(RegistryError::EmptyPage(page.id.0.clone()));
                }
                for section in &page.sections {
                    if section.settings.is_empty() {
                        return Err(RegistryError::EmptySection {
                            page: page.id.0.clone(),
                            section: section.id.clone(),
                        });
                    }
                    for setting in &section.settings {
                        if !seen_settings.insert(setting.id.clone()) {
                            return Err(RegistryError::DuplicateSetting(setting.id.0.clone()));
                        }
                        if setting.page_id != page.id || setting.section_id != section.id {
                            return Err(RegistryError::DuplicateSetting(format!(
                                "{} (misplaced in page {} section {})",
                                setting.id.0, page.id.0, section.id
                            )));
                        }
                        self.validate_binding(&setting.id, &setting.read.backend)?;
                        if let Some(write) = &setting.write {
                            self.validate_binding(&setting.id, &write.backend)?;
                        }
                    }
                }
            }
        }

        Ok(RegistrySnapshot {
            schema_version: REGISTRY_SCHEMA_VERSION,
            categories: self.categories,
        })
    }

    fn validate_binding(
        &self,
        setting: &SettingId,
        backend: &BackendId,
    ) -> Result<(), RegistryError> {
        if self.known_backends.contains(backend) {
            Ok(())
        } else {
            Err(RegistryError::UnknownBackend {
                setting: setting.0.clone(),
                backend: backend.0.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{ConfirmationPolicy, OperationKey};
    use crate::capability::PolkitActionId;

    fn sample_setting(page: &PageId, section: &str) -> SettingDefinition {
        SettingDefinition {
            id: SettingId(String::from("timedate.clock.use-ntp")),
            metadata: SettingMetadata {
                title: I18nKey(String::from("settings.timedate.ntp.title")),
                description: Some(I18nKey(String::from("settings.timedate.ntp.desc"))),
                aliases: vec![String::from("network time")],
                keywords: vec![String::from("clock")],
                search_weight: 0,
            },
            page_id: page.clone(),
            section_id: String::from(section),
            control: ControlKind::Toggle,
            read: ReadBinding {
                backend: BackendId::timedate(),
                key: crate::backend::ValueKey(String::from("NTP")),
                poll: crate::backend::PollPolicy::OnDemand,
            },
            write: Some(WriteAction {
                backend: BackendId::timedate(),
                operation: OperationKey(String::from("SetNtp")),
                confirmation: ConfirmationPolicy::None,
            }),
            permission: CapabilityRequirement::Polkit {
                action: PolkitActionId(String::from("org.zyntrix.zettings.timedate.set-ntp")),
            },
            hardware_dependency: None,
            reboot_hint: RebootHint::None,
            default_value: Some(SettingValue::Bool(true)),
        }
    }

    fn sample_category() -> Category {
        let page_id = PageId(String::from("datetime"));
        let setting = sample_setting(&page_id, "clock");
        Category {
            id: CategoryId(String::from("system")),
            title: I18nKey(String::from("categories.system")),
            icon: IconToken(String::from("laptop")),
            order: 1,
            keywords: vec![],
            availability: None,
            pages: vec![Page {
                id: page_id,
                category_id: CategoryId(String::from("system")),
                title: I18nKey(String::from("pages.datetime")),
                description: None,
                template: PageTemplate::SettingsPage,
                sections: vec![Section {
                    id: String::from("clock"),
                    title: Some(I18nKey(String::from("sections.clock"))),
                    settings: vec![setting],
                    visibility: None,
                }],
                required_capabilities: vec![],
            }],
        }
    }

    #[test]
    fn build_accepts_valid_registry_and_counts() {
        let mut builder = RegistryBuilder::new(&[BackendId::timedate()]);
        builder.register_category(sample_category()).unwrap();
        let snapshot = builder.build().unwrap();
        assert_eq!(snapshot.schema_version, REGISTRY_SCHEMA_VERSION);
        assert_eq!(snapshot.page_count(), 1);
        assert_eq!(snapshot.setting_count(), 1);
        assert!(
            snapshot
                .find_page(&PageId(String::from("datetime")))
                .is_some()
        );
    }

    #[test]
    fn build_rejects_unknown_backend_and_duplicates() {
        let mut missing_backend = RegistryBuilder::new(&[]);
        missing_backend
            .register_category(sample_category())
            .unwrap();
        let err = missing_backend.build();
        assert!(matches!(err, Err(RegistryError::UnknownBackend { .. })));

        let mut duplicate = RegistryBuilder::new(&[BackendId::timedate()]);
        duplicate.register_category(sample_category()).unwrap();
        assert!(duplicate.register_category(sample_category()).is_err());
    }

    #[test]
    fn page_route_resolves_to_itself() {
        let mut builder = RegistryBuilder::new(&[BackendId::timedate()]);
        builder.register_category(sample_category()).unwrap();
        let snapshot = builder.build().unwrap();
        let page = snapshot
            .find_page(&PageId(String::from("datetime")))
            .unwrap();
        let route = page.route();
        assert_eq!(route.canonical(), "/system/datetime");
    }
}
