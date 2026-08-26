use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use zettings_core::ZettingsError;

use crate::gateway::Decision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditOutcome {
    Applied,
    Failed,
    Denied,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditRecord {
    pub ts_unix_secs: u64,
    pub actor: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting: Option<String>,
    pub outcome: AuditOutcome,
    pub decision: Decision,
}

pub struct AuditWriter {
    path: PathBuf,
}

impl AuditWriter {
    /// # Errors
    ///
    /// Returns [`ZettingsError::Io`] when the parent directory cannot be
    /// created or the file cannot be opened for appending.
    pub fn create(path: &Path) -> Result<Self, ZettingsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        File::options()
            .create(true)
            .append(true)
            .open(path)
            .map_err(ZettingsError::from)?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// # Errors
    ///
    /// Returns [`ZettingsError::Io`] when the record cannot be serialized or
    /// flushed to disk.
    pub fn append(&self, record: &AuditRecord) -> Result<(), ZettingsError> {
        let mut line = serde_json::to_string(record).map_err(|_| ZettingsError::NotSupported {
            reason: zettings_core::UnsupportedReason::FeatureNotCompiled,
        })?;
        line.push('\n');
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        file.write_all(line.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zettings-audit-{tag}-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ))
    }

    #[test]
    fn appends_jsonl_records_atomically_per_line() {
        let path = temp_path("append");
        let writer = AuditWriter::create(&path).unwrap();

        writer
            .append(&AuditRecord {
                ts_unix_secs: 1_787_000_001,
                actor: String::from("zyntrix"),
                action: String::from("org.zyntrix.zettings.timedate.set-ntp"),
                setting: Some(String::from("timedate.clock.use-ntp")),
                outcome: AuditOutcome::Applied,
                decision: Decision::Allowed,
            })
            .unwrap();
        writer
            .append(&AuditRecord {
                ts_unix_secs: 1_787_000_002,
                actor: String::from("zyntrix"),
                action: String::from("org.zyntrix.zettings.network.forget"),
                setting: None,
                outcome: AuditOutcome::Denied,
                decision: Decision::Denied,
            })
            .unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(
            serde_json::from_str::<AuditRecord>(lines[0])
                .unwrap()
                .setting
                .is_some()
        );
        assert!(std::fs::remove_file(&path).is_ok());
    }

    #[test]
    fn create_builds_missing_parent_directories() {
        let base = temp_path("dirs");
        let path = base.join("nested").join("audit.jsonl");
        let writer = AuditWriter::create(&path).unwrap();
        assert!(path.exists());
        assert!(
            writer
                .append(&AuditRecord {
                    ts_unix_secs: 0,
                    actor: String::from("a"),
                    action: String::from("org.zyntrix.zettings.x"),
                    setting: None,
                    outcome: AuditOutcome::Failed,
                    decision: Decision::AuthenticationRequired,
                })
                .is_ok()
        );
        let _ = std::fs::remove_dir_all(base);
    }
}
