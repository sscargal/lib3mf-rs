use crate::error::{Lib3mfError, Result};
use rsa::{
    Oaep,
    RsaPrivateKey,
    RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey},
    pkcs8::DecodePublicKey, // Ensure this is used
};
// Removed DecodePrivateKey from pkcs8 usage on PublicKey lines to avoid confusion.
// Actually RsaPrivateKey needs DecodePrivateKey from pkcs8 too.
use rand::rngs::OsRng;
use rsa::pkcs8::DecodePrivateKey;
use sha1::Sha1;
use std::fs;
use std::path::Path;

pub struct KeyManager;

impl KeyManager {
    /// Loads an RSA private key from a PEM file.
    pub fn load_private_key<P: AsRef<Path>>(path: P) -> Result<RsaPrivateKey> {
        let content = fs::read_to_string(path)
            .map_err(|e| Lib3mfError::Validation(format!("Failed to read key file: {}", e)))?;

        // Try PKCS#1 then PKCS#8
        RsaPrivateKey::from_pkcs1_pem(&content)
            .or_else(|_| RsaPrivateKey::from_pkcs8_pem(&content))
            .map_err(|e| Lib3mfError::Validation(format!("Invalid private key format: {}", e)))
    }

    /// Loads an RSA public key from a PEM file.
    pub fn load_public_key<P: AsRef<Path>>(path: P) -> Result<RsaPublicKey> {
        let content = fs::read_to_string(path)
            .map_err(|e| Lib3mfError::Validation(format!("Failed to read key file: {}", e)))?;

        RsaPublicKey::from_pkcs1_pem(&content)
            .or_else(|_| RsaPublicKey::from_public_key_pem(&content))
            .map_err(|e| Lib3mfError::Validation(format!("Invalid public key format: {}", e)))
    }

    /// Wraps (encrypts) a CEK using the public key (KEK).
    /// Uses RSA-OAEP with SHA-1 digest (per 3MF spec).
    pub fn wrap_key(public_key: &RsaPublicKey, cek: &[u8]) -> Result<Vec<u8>> {
        let mut rng = OsRng;
        let padding = Oaep::new::<Sha1>();
        public_key
            .encrypt(&mut rng, padding, cek)
            .map_err(|e| Lib3mfError::EncryptionError(format!("Key wrapping failed: {}", e)))
    }

    /// Unwraps (decrypts) a CEK using the private key (KEK).
    /// Uses RSA-OAEP with SHA-1 digest.
    pub fn unwrap_key(private_key: &RsaPrivateKey, wrapped_key: &[u8]) -> Result<Vec<u8>> {
        let padding = Oaep::new::<Sha1>();
        private_key
            .decrypt(padding, wrapped_key)
            .map_err(|e| Lib3mfError::EncryptionError(format!("Key unwrapping failed: {}", e)))
    }
}
