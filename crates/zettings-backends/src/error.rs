use zettings_core::{BackendId, ZettingsError};

#[must_use]
pub fn to_zettings_error(backend: &BackendId, err: &zbus::Error) -> ZettingsError {
    ZettingsError::BackendUnreachable {
        backend: backend.clone(),
        detail: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zbus_errors_map_to_backend_unreachable() {
        let err = zbus::Error::NameTaken;
        let mapped = to_zettings_error(&BackendId::timedate(), &err);
        assert_eq!(mapped.code(), "backend-unreachable");
        assert!(mapped.to_string().contains("timedate"));
    }
}
