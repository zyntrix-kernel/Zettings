//! System audio routing, master volume, and sink models.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// System master audio routing and volume status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AudioStatus {
    pub master_volume: f64,
    pub muted: bool,
    pub default_sink: String,
    pub default_source: String,
}
