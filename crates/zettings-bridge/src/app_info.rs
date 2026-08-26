#[cxx_qt::bridge(namespace = "org::zyntrix::zettings")]
mod app_info {
    extern "RustQt" {
        #[qobject]
        type AppInfoBridge = super::AppInfoBridgeRust;

        #[qinvokable]
        fn version_major(self: &AppInfoBridge) -> u32;

        #[qinvokable]
        fn version_minor(self: &AppInfoBridge) -> u32;

        #[qinvokable]
        fn version_patch(self: &AppInfoBridge) -> u32;
    }
}

pub struct AppInfoBridgeRust;

impl AppInfoBridgeRust {
    pub fn version_major(&self) -> u32 {
        env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0)
    }

    pub fn version_minor(&self) -> u32 {
        env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0)
    }

    pub fn version_patch(&self) -> u32 {
        env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_workspace_crate_version() {
        let info = AppInfoBridgeRust;
        assert_eq!(info.version_major(), 0);
        assert_eq!(info.version_minor(), 1);
        assert_eq!(info.version_patch(), 0);
    }
}
