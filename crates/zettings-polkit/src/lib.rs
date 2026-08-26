//! Authorization gateway and audit log for privileged Zettings actions.
//!
//! This crate is the ONLY component permitted to consult `PolicyKit1`; backend
//! adapters receive pre-authorized operations (see docs/architecture/
//! capability-model.md). The live D-Bus transport to org.freedesktop.PolicyKit1
//! lands with the bridge integration increment; the gateway contract, the
//! dev-sandbox allow-list gateway, and the append-only audit log are complete
//! and tested here.

mod audit;
mod gateway;

pub use audit::{AuditOutcome, AuditRecord, AuditWriter};
pub use gateway::{AllowListGateway, AuthorizationQuery, Decision, Gateway};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_ranking_is_total() {
        assert!(Decision::Allowed > Decision::AuthenticationRequired);
        assert!(Decision::AuthenticationRequired > Decision::Denied);
    }

    #[test]
    fn audit_record_survives_json_round_trip() {
        let record = AuditRecord {
            ts_unix_secs: 1_787_000_000,
            actor: String::from("zyntrix"),
            action: String::from("org.zyntrix.zettings.timedate.set-ntp"),
            setting: Some(String::from("timedate.clock.use-ntp")),
            outcome: AuditOutcome::Applied,
            decision: Decision::Allowed,
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: AuditRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }
}
