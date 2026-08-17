//! Plugin discovery — scans the system and user module directories, parses
//! each module's `module.toml`, and verifies its `module.sig` against the
//! pinned Zyntrix Ed25519 public key. Verified modules are returned to the
//! caller for registration in [`crate::Registry`].
//!
//! Directory layout (per module):
//! ```text
//! /usr/lib/zyntrix/zettings/modules/<module-id>/
//!   ├─ module.toml      # Manifest (parsed by [`crate::manifest`]).
//!   └─ module.sig       # Ed25519 signature over module.toml bytes.
//! ```
//!
//! During the `zettings-mock` dev loop (Windows host) the verifier still runs
//! but the user-namespace directory falls back to `~/.local/share/...` and
//! missing signature files are tolerated — modules are signed before they
//! ship to the system dir, not during local authoring.

use crate::manifest::ModuleManifest;
use crate::signature::{PublicKey, SignatureError};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors surfaced by the discovery loader.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// The module's `module.toml` could not be read or decoded as UTF-8.
    #[error("manifest unreadable in {0}: {1}")]
    ManifestUnreadable(PathBuf, String),
    /// The module's `module.toml` could not be parsed.
    #[error("manifest parse error in {path}: {source}")]
    Manifest {
        /// Filesystem path to the offending `module.toml`.
        path: PathBuf,
        /// Underlying `toml::de::Error`.
        #[source]
        source: toml::de::Error,
    },
    /// The module's `module.sig` could not be read.
    #[error("signature unreadable in {0}")]
    SignatureUnreadable(PathBuf),
    /// The signature did not verify.
    #[error("signature verification failed in {0}")]
    Signature(#[from] SignatureError),
    /// A directory under a module search root was not a directory.
    #[error("not a directory: {0}")]
    NotADirectory(PathBuf),
}

/// The result of discovering a single module.
#[derive(Debug, Clone)]
pub struct DiscoveredModule {
    /// The parsed and validated manifest.
    pub manifest: ModuleManifest,
    /// The directory the module lives in.
    pub path: PathBuf,
}

/// Load every module under `root`, verifying each `module.sig` against `key`.
///
/// `root` is expected to be a directory containing one subdirectory per
/// module. Any subdir that lacks `module.toml` or `module.sig` is skipped
/// silently — the discovery pass is forgiving of leftover non-module dirs.
///
/// # Errors
/// - [`DiscoveryError::NotADirectory`] when `root` exists but is not a dir.
/// - Per-module errors are collected and returned in unspecified order; the
///   loader does not abort the whole pass on a single bad module.
#[allow(clippy::result_large_err)]
pub fn discover(
    root: &Path,
    key: &PublicKey,
) -> Result<Vec<DiscoveredModule>, Vec<DiscoveryError>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        return Err(vec![DiscoveryError::NotADirectory(root.to_path_buf())]);
    }

    let mut ok = Vec::new();
    let mut errs = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return Err(vec![DiscoveryError::NotADirectory(root.to_path_buf())]);
    };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        match load_module(&dir, key) {
            Ok(module) => ok.push(module),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() { Ok(ok) } else { Err(errs) }
}

/// Load and verify a single module from `dir`.
#[allow(clippy::result_large_err)]
fn load_module(dir: &Path, key: &PublicKey) -> Result<DiscoveredModule, DiscoveryError> {
    let manifest_path = dir.join("module.toml");
    let sig_path = dir.join("module.sig");
    let bytes = std::fs::read(&manifest_path)
        .map_err(|e| DiscoveryError::ManifestUnreadable(manifest_path.clone(), e.to_string()))?;
    let manifest_str = std::str::from_utf8(&bytes).map_err(|_| {
        DiscoveryError::ManifestUnreadable(manifest_path.clone(), "non-utf8".to_string())
    })?;
    let manifest: ModuleManifest =
        toml::from_str(manifest_str).map_err(|source| DiscoveryError::Manifest {
            path: manifest_path.clone(),
            source,
        })?;
    let sig = std::fs::read(&sig_path)
        .map_err(|_| DiscoveryError::SignatureUnreadable(sig_path.clone()))?;
    key.verify(&manifest.id.0, manifest_str.as_bytes(), &sig)?;
    Ok(DiscoveredModule {
        manifest,
        path: dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ModuleId;
    use ring::rand::SystemRandom;
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use std::fs;
    use tempfile::tempdir;

    fn pinned_key() -> (PublicKey, Ed25519KeyPair) {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate");
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("parse");
        let key = PublicKey::from_bytes(pair.public_key().as_ref().to_vec());
        (key, pair)
    }

    fn write_module(dir: &Path, id: &str, pair: &Ed25519KeyPair) -> PathBuf {
        let module_dir = dir.join(id);
        fs::create_dir_all(&module_dir).expect("mkdir");
        let toml = format!(
            r#"id = "{id}"
version = "0.1.0"
name = "Sample"
icon = "monitor"
route = "/x"
"#
        );
        let sig = pair.sign(toml.as_bytes());
        fs::write(module_dir.join("module.toml"), &toml).expect("write toml");
        fs::write(module_dir.join("module.sig"), sig.as_ref()).expect("write sig");
        module_dir
    }

    #[test]
    fn discovers_signed_module() {
        let (key, pair) = pinned_key();
        let root = tempdir().expect("tempdir");
        let _ = write_module(root.path(), "org.zyntrix.zettings.display", &pair);
        let found = discover(root.path(), &key).expect("ok");
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].manifest.id,
            ModuleId::new("org.zyntrix.zettings.display")
        );
    }

    #[test]
    fn skips_module_with_missing_sig() {
        let (key, _pair) = pinned_key();
        let root = tempdir().expect("tempdir");
        let module_dir = root.path().join("org.zyntrix.zettings.audio");
        fs::create_dir_all(&module_dir).expect("mkdir");
        fs::write(
            module_dir.join("module.toml"),
            r#"id = "org.zyntrix.zettings.audio"
version = "0.1.0"
name = "Audio"
icon = "volume-2"
route = "/audio"
"#,
        )
        .expect("write toml");
        let errs = discover(root.path(), &key).expect_err("errs");
        assert!(
            errs.iter()
                .any(|e| matches!(e, DiscoveryError::SignatureUnreadable(_)))
        );
    }

    #[test]
    fn rejects_tampered_manifest() {
        let (key, pair) = pinned_key();
        let root = tempdir().expect("tempdir");
        let module_dir = write_module(root.path(), "org.zyntrix.zettings.network", &pair);
        // Tamper with the manifest after writing the signature.
        fs::write(
            module_dir.join("module.toml"),
            r#"id = "org.zyntrix.zettings.network"
version = "0.2.0-tampered"
name = "Network"
icon = "wifi"
route = "/network"
"#,
        )
        .expect("rewrite");
        let errs = discover(root.path(), &key).expect_err("errs");
        assert!(
            errs.iter()
                .any(|e| matches!(e, DiscoveryError::Signature(_)))
        );
    }

    #[test]
    fn empty_dir_yields_empty_result() {
        let (key, _pair) = pinned_key();
        let root = tempdir().expect("tempdir");
        let found = discover(root.path(), &key).expect("ok");
        assert!(found.is_empty());
    }

    #[test]
    fn nonexistent_root_yields_empty_result() {
        let (key, _pair) = pinned_key();
        let root = tempdir().expect("tempdir");
        let absent = root.path().join("does-not-exist");
        let found = discover(&absent, &key).expect("ok");
        assert!(found.is_empty());
    }
}
