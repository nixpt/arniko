//! Node identity based on Ed25519 keys
//!
//! Each node has a unique identity derived from its keypair.
//! This identity is used for:
//! - P2P addressing (PeerId)
//! - DID generation (did:key:...)
//! - Authentication (WBPA challenge-response)

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Node identity containing keypair
pub struct NodeIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    did: String,
}

impl NodeIdentity {
    /// Generate a new random identity
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let did = Self::derive_did(&verifying_key);

        Self {
            signing_key,
            verifying_key,
            did,
        }
    }

    /// Load identity from file or generate new one
    pub fn load_or_generate(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let bytes = std::fs::read(path)?;
            if bytes.len() != 32 {
                anyhow::bail!("Invalid key file length");
            }
            let key_bytes: [u8; 32] = bytes.try_into().unwrap();
            let signing_key = SigningKey::from_bytes(&key_bytes);
            let verifying_key = signing_key.verifying_key();
            let did = Self::derive_did(&verifying_key);

            Ok(Self {
                signing_key,
                verifying_key,
                did,
            })
        } else {
            let identity = Self::generate();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, identity.signing_key.to_bytes())?;
            Ok(identity)
        }
    }

    /// Derive DID from public key (did:key method)
    fn derive_did(verifying_key: &VerifyingKey) -> String {
        // Multicodec prefix for Ed25519 public key: 0xed01
        let mut bytes = vec![0xed, 0x01];
        bytes.extend_from_slice(verifying_key.as_bytes());

        let encoded = bs58::encode(&bytes).into_string();
        format!("did:key:z{}", encoded)
    }

    /// Get the DID for this node
    pub fn did(&self) -> &str {
        &self.did
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Get a short fingerprint for display
    pub fn fingerprint(&self) -> String {
        let bytes = self.verifying_key.as_bytes();
        bs58::encode(&bytes[..8]).into_string()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    /// Verify a signature from another node
    pub fn verify(public_key: &[u8; 32], message: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }

        let Ok(verifying_key) = VerifyingKey::from_bytes(public_key) else {
            return false;
        };

        let sig_bytes: [u8; 64] = signature.try_into().unwrap();
        let signature = Signature::from_bytes(&sig_bytes);

        verifying_key.verify(message, &signature).is_ok()
    }

    /// Parse a DID to extract the public key
    pub fn did_to_public_key(did: &str) -> Option<[u8; 32]> {
        if !did.starts_with("did:key:z") {
            return None;
        }

        let encoded = &did[9..]; // Skip "did:key:z"
        let bytes = bs58::decode(encoded).into_vec().ok()?;

        // Check multicodec prefix
        if bytes.len() < 34 || bytes[0] != 0xed || bytes[1] != 0x01 {
            return None;
        }

        let key_bytes: [u8; 32] = bytes[2..34].try_into().ok()?;
        Some(key_bytes)
    }

    #[cfg(feature = "p2p")]
    /// Convert to libp2p PeerId
    pub fn to_peer_id(&self) -> libp2p::PeerId {
        use libp2p::identity::{ed25519, Keypair};

        let secret = ed25519::SecretKey::try_from_bytes(self.signing_key.to_bytes().to_vec())
            .expect("valid ed25519 key");

        let keypair = Keypair::from(ed25519::Keypair::from(secret));
        keypair.public().to_peer_id()
    }

    #[cfg(feature = "p2p")]
    /// Get libp2p Keypair
    pub fn to_libp2p_keypair(&self) -> libp2p::identity::Keypair {
        use libp2p::identity::{ed25519, Keypair};

        let secret = ed25519::SecretKey::try_from_bytes(self.signing_key.to_bytes().to_vec())
            .expect("valid ed25519 key");

        Keypair::from(ed25519::Keypair::from(secret))
    }

    /// Get the libp2p PeerId derived from this identity's keypair
    #[cfg(feature = "p2p")]
    pub fn peer_id(&self) -> String {
        self.to_libp2p_keypair().public().to_peer_id().to_string()
    }
}

impl std::fmt::Debug for NodeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeIdentity")
            .field("did", &self.did)
            .field("fingerprint", &self.fingerprint())
            .finish()
    }
}

/// Challenge for WBPA authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub nonce: String,
    pub timestamp: i64,
    pub domain: String,
    pub purpose: String,
}

impl Challenge {
    /// Create a new challenge for authentication
    pub fn new(domain: impl Into<String>) -> Self {
        use rand::Rng;
        let nonce: [u8; 16] = rand::thread_rng().gen();

        Self {
            nonce: bs58::encode(nonce).into_string(),
            timestamp: chrono::Utc::now().timestamp(),
            domain: domain.into(),
            purpose: "mesh-auth".to_string(),
        }
    }

    /// Serialize for signing
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Check if challenge is still valid (not expired)
    pub fn is_valid(&self, max_age_secs: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - self.timestamp).abs() < max_age_secs
    }
}

/// Signed challenge response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
    pub signature: String,  // base58 encoded
    pub public_key: String, // base58 encoded
    pub did: String,
}

impl ChallengeResponse {
    /// Create a signed response to a challenge
    pub fn sign(challenge: Challenge, identity: &NodeIdentity) -> Self {
        let signable = challenge.to_signable_bytes();
        let signature = identity.sign(&signable);

        Self {
            challenge,
            signature: bs58::encode(&signature).into_string(),
            public_key: bs58::encode(identity.public_key_bytes()).into_string(),
            did: identity.did().to_string(),
        }
    }

    /// Verify the response
    pub fn verify(&self) -> bool {
        let Ok(sig_bytes) = bs58::decode(&self.signature).into_vec() else {
            return false;
        };

        let Ok(pubkey_bytes) = bs58::decode(&self.public_key).into_vec() else {
            return false;
        };

        if pubkey_bytes.len() != 32 {
            return false;
        }

        let pubkey: [u8; 32] = pubkey_bytes.try_into().unwrap();
        let signable = self.challenge.to_signable_bytes();

        NodeIdentity::verify(&pubkey, &signable, &sig_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let id = NodeIdentity::generate();
        assert!(id.did().starts_with("did:key:z"));
        assert!(!id.fingerprint().is_empty());
    }

    #[test]
    fn test_sign_verify() {
        let id = NodeIdentity::generate();
        let message = b"hello world";
        let signature = id.sign(message);

        assert!(NodeIdentity::verify(
            &id.public_key_bytes(),
            message,
            &signature
        ));
        assert!(!NodeIdentity::verify(
            &id.public_key_bytes(),
            b"wrong",
            &signature
        ));
    }

    #[test]
    fn test_did_roundtrip() {
        let id = NodeIdentity::generate();
        let pubkey = NodeIdentity::did_to_public_key(id.did()).unwrap();
        assert_eq!(pubkey, id.public_key_bytes());
    }

    #[test]
    fn test_challenge_response() {
        let id = NodeIdentity::generate();
        let challenge = Challenge::new("test.exosphere.local");
        let response = ChallengeResponse::sign(challenge, &id);

        assert!(response.verify());
        assert!(response.challenge.is_valid(60));
    }
}
