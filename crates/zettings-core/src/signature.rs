//! Ed25519 plugin signature verification.
//!
//! Every discovered module ships a `module.sig` file alongside its
//! `module.toml`. The `.sig` is an Ed25519 signature over the canonical bytes
//! of `module.toml` produced by a Zyntrix-controlled key whose public half is
//! pinned into the shell at build time. A module that fails verification is
//! refused a mount — no exceptions, no warning override.

/// Errors surfaced by the signature verifier.
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    /// The `.sig` file was missing, the wrong length, or undecodable.
    #[error("invalid signature bytes: {0}")]
    InvalidSignature(String),
    /// The signature did not verify against the pinned public key.
    #[error("signature verification failed for module {0}")]
    VerificationFailed(String),
}

/// A verified Ed25519 public key. Constructed from the pinned Zyntrix
/// root key bytes; the key itself never travels over IPC.
#[derive(Debug, Clone)]
pub struct PublicKey {
    inner: ring::signature::UnparsedPublicKey<Vec<u8>>,
}

impl PublicKey {
    /// Construct a `PublicKey` from 32 raw Ed25519 public key bytes.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            inner: ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, bytes),
        }
    }

    /// Verify `signature` was produced by this key over `message`.
    ///
    /// # Errors
    /// - [`SignatureError::InvalidSignature`] when the signature is the wrong
    ///   length or malformed.
    /// - [`SignatureError::VerificationFailed`] when the signature does not
    ///   verify against this key.
    pub fn verify(
        &self,
        module_id: &str,
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), SignatureError> {
        // Ed25519 signatures are 64 bytes; surface a clean error rather than
        // the ring-internal panic when callers hand us a truncated file.
        if signature.len() != 64 {
            return Err(SignatureError::InvalidSignature(format!(
                "expected 64 bytes, got {}",
                signature.len()
            )));
        }
        self.inner
            .verify(message, signature)
            .map_err(|_| SignatureError::VerificationFailed(module_id.to_string()))
    }
}

// Tests import `ring::signature::Ed25519KeyPair` directly — it is private
// within `ring` and cannot be re-exported through this crate. Tests that
// mint signatures also pull in `ring::signature::KeyPair` for `public_key()`.

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn keypair() -> (Vec<u8>, Vec<u8>, Ed25519KeyPair) {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate");
        let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("parse");
        let public = pair.public_key().as_ref().to_vec();
        let private = pkcs8.as_ref().to_vec();
        (public, private, pair)
    }

    #[test]
    fn verifies_valid_signature() {
        let (public, _private, pair) = keypair();
        let key = PublicKey::from_bytes(public);
        let message = b"canonical module.toml bytes";
        let sig = pair.sign(message);
        key.verify("org.example.x", message, sig.as_ref())
            .expect("verifies");
    }

    #[test]
    fn rejects_tampered_message() {
        let (public, _private, pair) = keypair();
        let key = PublicKey::from_bytes(public);
        let message = b"canonical module.toml bytes";
        let sig = pair.sign(message);
        let err = key
            .verify("org.example.x", b"tampered", sig.as_ref())
            .unwrap_err();
        assert!(matches!(err, SignatureError::VerificationFailed(_)));
    }

    #[test]
    fn rejects_short_signature() {
        let (public, _private, _pair) = keypair();
        let key = PublicKey::from_bytes(public);
        let err = key
            .verify("org.example.x", b"msg", b"too short")
            .unwrap_err();
        assert!(matches!(err, SignatureError::InvalidSignature(_)));
    }

    #[test]
    fn round_trip_with_test_private_key() {
        // Demonstrate the helper re-export works for the registry discovery
        // tests, which mint a real signature from a generated keypair.
        let (public, private, _pair) = keypair();
        let key = PublicKey::from_bytes(public);
        let pair = Ed25519KeyPair::from_pkcs8(&private).expect("parse");
        let message = b"abc";
        let sig = pair.sign(message);
        key.verify("test", message, sig.as_ref()).expect("verify");
    }
}
