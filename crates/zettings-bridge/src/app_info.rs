#[cxx_qt::bridge(namespace = "org::zyntrix::zettings")]
mod app_info {
    #[namespace = ""]
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        #[qobject]
        type AppInfoBridge = super::AppInfoBridgeRust;

        #[qinvokable]
        fn app_name(self: &AppInfoBridge) -> QString;

        #[qinvokable]
        fn version(self: &AppInfoBridge) -> QString;
    }
}

pub struct AppInfoBridgeRust;

impl AppInfoBridgeRust {
    pub fn app_name(&self) -> QString {
        QString::from("Zettings")
    }

    pub fn version(&self) -> QString {
        QString::from(env!("CARGO_PKG_VERSION"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_workspace_crate_version() {
        assert_eq!(AppInfoBridgeRust.version().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn app_name_is_stable() {
        assert_eq!(AppInfoBridgeRust.app_name().to_string(), "Zettings");
    }
}
