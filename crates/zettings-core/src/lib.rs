//! Domain model for ZETTINGS.
//!
//! This crate is the bottom layer of the dependency DAG: it depends on no
//! other workspace crate and defines the vocabulary every other layer uses —
//! settings definitions ([`SettingDefinition`]), stable route identifiers
//! ([`RouteId`]), category identifiers ([`CategoryId`]), honest backend
//! capability reporting ([`CapabilityState`]), and the shared error type
//! ([`ZettingsError`]).
//!
//! The canonical settings graph flows downward:
//! `registry -> category -> page -> section -> setting`, as specified by
//! `PLAN.md` §3 and the Windows reconstruction specification §19.

mod capability;
mod error;
mod registry;
mod route;

pub use capability::CapabilityState;
pub use error::ZettingsError;
pub use registry::{
    BUILT_IN_CATEGORY_IDS, CategoryId, CategorySummary, ControlType, SettingDefinition,
    ValueRequirement, built_in_categories, built_in_page_definitions,
};
pub use route::{ROUTE_SCHEME, RouteId};
