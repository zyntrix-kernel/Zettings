//! Local system user account model.

use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Local system user account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct UserInfo {
    pub username: String,
    pub real_name: String,
    pub is_admin: bool,
    pub home_dir: String,
    pub shell: String,
}
