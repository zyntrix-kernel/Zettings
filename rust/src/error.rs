//! Typed backend errors with lossless D-Bus mapping.
//!
//! Every fallible backend operation returns [`BackendError`]. Over D-Bus the
//! kind travels twice: once as the standard `org.freedesktop.DBus` variant
//! (for generic clients) and once as a `code: detail` message prefix that the
//! Dart side parses back into the exact kind. Wire format is therefore:
//! `<code>: <human message>`, e.g. `permissionDenied: polkit auth required`.

use std::fmt;
use zbus::fdo;

/// Machine-readable error kinds. Codes mirror the Dart `AppErrorType` names
/// so the Flutter side recovers them without a lookup table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unavailable,
    Unsupported,
    PermissionDenied,
    InvalidInput,
    Timeout,
    ServiceUnavailable,
    HardwareUnavailable,
    SystemError,
}

impl ErrorKind {
    /// Stable wire code. Never rename: Dart parses this prefix.
    pub fn as_code(self) -> &'static str {
        match self {
            ErrorKind::Unavailable => "unavailable",
            ErrorKind::Unsupported => "unsupported",
            ErrorKind::PermissionDenied => "permissionDenied",
            ErrorKind::InvalidInput => "invalidInput",
            ErrorKind::Timeout => "timeout",
            ErrorKind::ServiceUnavailable => "serviceUnavailable",
            ErrorKind::HardwareUnavailable => "hardwareUnavailable",
            ErrorKind::SystemError => "systemError",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_code())
    }
}

/// A typed backend failure: a machine kind plus a human message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    kind: ErrorKind,
    message: String,
}

impl BackendError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for BackendError {}

/// Maps each kind onto the closest standard D-Bus error. Kinds without an
/// exact standard counterpart become `Failed`; the kind is never lost because
/// it stays in the `code: detail` message.
impl From<BackendError> for fdo::Error {
    fn from(error: BackendError) -> Self {
        let message = error.to_string();
        match error.kind {
            ErrorKind::Unsupported => fdo::Error::NotSupported(message),
            ErrorKind::PermissionDenied => fdo::Error::AccessDenied(message),
            ErrorKind::InvalidInput => fdo::Error::InvalidArgs(message),
            ErrorKind::Timeout => fdo::Error::TimedOut(message),
            ErrorKind::Unavailable
            | ErrorKind::ServiceUnavailable
            | ErrorKind::HardwareUnavailable
            | ErrorKind::SystemError => fdo::Error::Failed(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format_is_code_colon_detail() {
        let error = BackendError::new(ErrorKind::PermissionDenied, "nope");
        let rendered = error.to_string();
        assert_eq!(rendered, "permissionDenied: nope");
        let (code, _) = rendered.split_once(": ").unwrap();
        assert_eq!(code, ErrorKind::PermissionDenied.as_code());
    }

    #[test]
    fn every_kind_maps_to_a_dbus_error_and_keeps_its_code() {
        let kinds = [
            ErrorKind::Unavailable,
            ErrorKind::Unsupported,
            ErrorKind::PermissionDenied,
            ErrorKind::InvalidInput,
            ErrorKind::Timeout,
            ErrorKind::ServiceUnavailable,
            ErrorKind::HardwareUnavailable,
            ErrorKind::SystemError,
        ];
        for kind in kinds {
            let dbus: fdo::Error = BackendError::new(kind, "detail").into();
            assert!(
                dbus.to_string().contains(kind.as_code()),
                "D-Bus message lost kind {kind:?}: {dbus}"
            );
        }
    }

    #[test]
    fn exact_standard_variants_are_used_where_they_exist() {
        let denied: fdo::Error = BackendError::new(ErrorKind::PermissionDenied, "x").into();
        assert!(matches!(denied, fdo::Error::AccessDenied(_)));
        let bad_input: fdo::Error = BackendError::new(ErrorKind::InvalidInput, "x").into();
        assert!(matches!(bad_input, fdo::Error::InvalidArgs(_)));
        let unsupported: fdo::Error = BackendError::new(ErrorKind::Unsupported, "x").into();
        assert!(matches!(unsupported, fdo::Error::NotSupported(_)));
        let timeout: fdo::Error = BackendError::new(ErrorKind::Timeout, "x").into();
        assert!(matches!(timeout, fdo::Error::TimedOut(_)));
    }
}
