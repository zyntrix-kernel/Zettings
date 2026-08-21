//! Stable deep-link routes (`zettings://` scheme).
//!
//! Rules (`PLAN.md` §6): every navigable setting has a stable route; every
//! route maps to exactly one page definition; routes are serializable and
//! testable; deep-linked pages must load directly even when backends fail.

use crate::error::ZettingsError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// URI scheme used for all ZETTINGS deep links.
pub const ROUTE_SCHEME: &str = "zettings://";

/// A validated `zettings://` route, e.g. `zettings://system/display`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RouteId(String);

impl RouteId {
    /// Validates and constructs a route.
    ///
    /// Grammar: scheme prefix + `/`-separated segments of `[a-z0-9-]`
    /// (the same slug grammar as category identifiers), no empty segments,
    /// no trailing slash.
    ///
    /// # Errors
    /// Returns [`ZettingsError::InvalidRoute`] when the grammar is violated.
    pub fn parse(raw: &str) -> Result<Self, ZettingsError> {
        let rest = raw.strip_prefix(ROUTE_SCHEME).ok_or_else(|| {
            ZettingsError::invalid_route(raw, format!("must start with {ROUTE_SCHEME}"))
        })?;
        if rest.is_empty() {
            return Err(ZettingsError::invalid_route(raw, "missing path"));
        }
        if rest.starts_with('/') || rest.ends_with('/') {
            return Err(ZettingsError::invalid_route(
                raw,
                "no leading/trailing slash",
            ));
        }
        for segment in rest.split('/') {
            if segment.is_empty() {
                return Err(ZettingsError::invalid_route(raw, "empty path segment"));
            }
            crate::registry::CategoryId::parse(segment).map_err(|_| {
                ZettingsError::invalid_route(raw, format!("bad segment {segment:?}"))
            })?;
        }
        Ok(Self(raw.to_owned()))
    }

    /// The full route string including the scheme.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path segments after the scheme.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.strip_prefix(ROUTE_SCHEME).unwrap_or("").split('/')
    }
}

impl fmt::Display for RouteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_routes() {
        for raw in [
            "zettings://system",
            "zettings://system/display",
            "zettings://updates",
        ] {
            let r = RouteId::parse(raw).unwrap_or_else(|e| panic!("{raw}: {e}"));
            assert_eq!(r.as_str(), raw);
        }
    }

    #[test]
    fn rejects_invalid_routes() {
        for raw in [
            "",
            "ms-settings:display",
            "zettings://",
            "zettings:///system",
            "zettings://System/Display",
            "zettings://system/",
            "zettings://sys tem",
        ] {
            assert!(RouteId::parse(raw).is_err(), "{raw:?} should be rejected");
        }
    }

    #[test]
    fn segments_split_paths() {
        let r = RouteId::parse("zettings://system/display").expect("valid");
        assert_eq!(r.segments().collect::<Vec<_>>(), vec!["system", "display"]);
    }
}
