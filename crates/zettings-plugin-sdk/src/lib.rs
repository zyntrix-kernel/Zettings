//! Manifest format and signature verification for ZETTINGS settings modules.
//!
//! Modules extend the settings graph with additional pages. Because modules
//! are effectively privileged configuration contributors (threat-model.md T8),
//! every module must carry an ed25519 signature over its canonical manifest
//! bytes, verified against a distribution public key before its definitions
//! are admitted into the registry.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Wire/manifest schema version understood by this SDK.
pub const MANIFEST_API_VERSION: u32 = 1;

/// Declared capabilities a module requests.
///
/// Capability names mirror backend adapter areas; the loader grants only
/// declared, signed, and policy-allowed capabilities — never ambient authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ModuleCapability {
    /// Contribute page/setting definitions to the registry.
    RegistryContribute,
    /// Read state from the named adapter area (payload: adapter id).
    AdapterRead(String),
}

/// The declarative manifest of a settings module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Stable module identifier, e.g. `org.zyntrix.module-gaming-extra`.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Module version (semver string).
    pub version: String,
    /// Must equal [`MANIFEST_API_VERSION`]; mismatched modules are rejected.
    pub api_version: u32,
    /// Requested capabilities; unsigned or unallowed entries are refused.
    #[serde(default)]
    pub capabilities: Vec<ModuleCapability>,
    /// Base64(ed25519 signature) over [`ModuleManifest::signing_payload`].
    pub signature: String,
}

impl ModuleManifest {
    /// Canonical byte payload covered by the signature: every field except
    /// `signature`, serialized deterministically via serde JSON (maps and
    /// enums in this struct are deterministic by construction).
    ///
    /// # Errors
    /// Returns [`ManifestError::Serialization`] when canonical serialization
    /// fails, which indicates a programmer error rather than user input.
    pub fn signing_payload(&self) -> Result<Vec<u8>, ManifestError> {
        let unsigned = UnsignedView {
            id: &self.id,
            name: &self.name,
            version: &self.version,
            api_version: self.api_version,
            capabilities: &self.capabilities,
        };
        serde_json::to_vec(&unsigned).map_err(|source| ManifestError::Serialization { source })
    }

    /// Verifies the manifest against a raw ed25519 public key (32 bytes).
    ///
    /// # Errors
    /// - [`ManifestError::UnsupportedApiVersion`] on version mismatch;
    /// - [`ManifestError::MalformedSignature`] when base64 decoding fails;
    /// - [`ManifestError::InvalidSignature`] when verification fails;
    /// - [`ManifestError::Serialization`] when the payload cannot be built.
    pub fn verify(&self, public_key: &[u8]) -> Result<(), ManifestError> {
        if self.api_version != MANIFEST_API_VERSION {
            return Err(ManifestError::UnsupportedApiVersion {
                found: self.api_version,
                expected: MANIFEST_API_VERSION,
            });
        }
        let sig = base64::engine::general_purpose::STANDARD
            .decode(self.signature.as_bytes())
            .map_err(|_| ManifestError::MalformedSignature)?;
        let payload = self.signing_payload()?;
        let key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
        key.verify(&payload, &sig)
            .map_err(|_| ManifestError::InvalidSignature)
    }
}

/// Signature-free projection used to build the canonical signing bytes.
#[derive(Serialize)]
struct UnsignedView<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
    api_version: u32,
    capabilities: &'a [ModuleCapability],
}

/// Errors produced during manifest validation and verification.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ManifestError {
    /// The manifest targets a different SDK generation.
    #[error("unsupported api version {found} (expected {expected})")]
    UnsupportedApiVersion {
        /// Version found in the manifest.
        found: u32,
        /// Version supported by this SDK ([`MANIFEST_API_VERSION`]).
        expected: u32,
    },
    /// The signature is not valid base64.
    #[error("malformed signature encoding")]
    MalformedSignature,
    /// The signature does not match the manifest contents.
    #[error("signature verification failed")]
    InvalidSignature,
    /// Canonical serialization of the unsigned view failed.
    #[error("manifest serialization failed")]
    Serialization {
        /// Underlying serde error.
        source: serde_json::Error,
    },
}

/// Convenience helper for tests/tools: signs a manifest payload with an
/// ed25519 keypair given as hex-encoded PKCS#8 v2 document.
///
/// Not part of the production loader path; distribution signing happens
/// offline. Kept `pub` because integration tests across crates need identical
/// signing semantics.
///
/// # Errors
/// Returns [`ManifestError::MalformedSignature`] when the key material is not
/// valid hex or not a usable ed25519 PKCS#8 document.
pub fn sign_payload_pkcs8_hex(
    payload: &[u8],
    pkcs8_secret_key_hex: &str,
) -> Result<String, ManifestError> {
    let der = hex::decode(pkcs8_secret_key_hex).map_err(|_| ManifestError::MalformedSignature)?;
    let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(&der)
        .map_err(|_| ManifestError::MalformedSignature)?;
    let sig = key_pair.sign(payload);
    Ok(base64::engine::general_purpose::STANDARD.encode(sig.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_key() -> (String, Vec<u8>) {
        use ring::signature::KeyPair as _;
        let rng = ring::rand::SystemRandom::new();
        let doc = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng).expect("keygen supported");
        let der = doc.as_ref().to_vec();
        let pair = ring::signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(&der).unwrap();
        (hex::encode(der), pair.public_key().as_ref().to_vec())
    }

    #[test]
    fn verify_accepts_correctly_signed_manifest() {
        let (sk_hex, pk) = generated_key();
        let mut manifest = ModuleManifest {
            id: "org.zyntrix.module-test".into(),
            name: "Test Module".into(),
            version: "0.1.0".into(),
            api_version: MANIFEST_API_VERSION,
            capabilities: vec![ModuleCapability::RegistryContribute],
            signature: String::new(),
        };
        let payload = manifest.signing_payload().expect("payload");
        manifest.signature = sign_payload_pkcs8_hex(&payload, &sk_hex).expect("signable");
        assert!(manifest.verify(&pk).is_ok());
    }

    #[test]
    fn verify_rejects_tampered_manifest() {
        let (sk_hex, pk) = generated_key();
        let mut manifest = ModuleManifest {
            id: "org.zyntrix.module-test".into(),
            name: "Test Module".into(),
            version: "0.1.0".into(),
            api_version: MANIFEST_API_VERSION,
            capabilities: vec![],
            signature: String::new(),
        };
        manifest.signature =
            sign_payload_pkcs8_hex(&manifest.signing_payload().unwrap(), &sk_hex).unwrap();
        manifest.name = "Tampered".into();
        assert!(matches!(
            manifest.verify(&pk),
            Err(ManifestError::InvalidSignature)
        ));
    }

    #[test]
    fn rejects_wrong_api_version() {
        let (_, pk) = generated_key();
        let manifest = ModuleManifest {
            id: "org.zyntrix.module-test".into(),
            name: "M".into(),
            version: "0.1.0".into(),
            api_version: 999,
            capabilities: vec![],
            signature: String::new(),
        };
        assert!(matches!(
            manifest.verify(&pk),
            Err(ManifestError::UnsupportedApiVersion {
                found: 999,
                expected: MANIFEST_API_VERSION,
            })
        ));
    }
}
