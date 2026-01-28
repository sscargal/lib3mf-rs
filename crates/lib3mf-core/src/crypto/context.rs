use crate::error::{Lib3mfError, Result};
use uuid::Uuid;
use crate::model::secure_content::*;
use crate::crypto::keys::KeyManager;
use crate::crypto::encryption::decrypt_aes256gcm;
use crate::archive::ArchiveReader;
// use rsa::RsaPrivateKey; 
// To avoid strict rsa dependency in public struct fields if possible, or use generic?
// For now, use RsaPrivateKey.
use rsa::RsaPrivateKey;
use std::collections::HashMap;

/// Manages secure content decryption context.
pub struct SecureContext {
    keystore: KeyStore,
    private_key: RsaPrivateKey,
    /// Cache of unwrapped CEKs (KeyUUID -> CEK bytes)
    cek_cache: HashMap<String, Vec<u8>>,
    /// Map of Resource Path to KeyUUID (derived from Relationships)
    resource_key_map: HashMap<String, Uuid>,
    /// Consumer ID we are acting as
    consumer_id: String,
}

impl SecureContext {
    pub fn new(
        keystore: KeyStore, 
        private_key: RsaPrivateKey, 
        resource_key_map: HashMap<String, Uuid>,
        consumer_id: String
    ) -> Self {
        Self {
            keystore,
            private_key,
            cek_cache: HashMap::new(),
            resource_key_map,
            consumer_id,
        }
    }
    
    /// Reads and decrypts an entry from the archive if it is covered by the KeyStore.
    /// If the path is not encrypted (not in map), returns None (or error if strictly checking?).
    /// Recommended: Return `Ok(Some(data))` if decrypted, `Ok(None)` if not encrypted, `Err` if failure.
    pub fn decrypt_entry(&mut self, archiver: &mut impl ArchiveReader, path: &str) -> Result<Option<Vec<u8>>> {
        let key_uuid = match self.resource_key_map.get(path) {
            Some(u) => u,
            None => return Ok(None), // Not encrypted
        };
        
        // Find Group
        let group = self.keystore.resource_data_groups.iter()
            .find(|g| g.key_uuid == *key_uuid)
            .ok_or_else(|| Lib3mfError::Validation(format!("KeyUUID {} not found in KeyStore", key_uuid)))?;
            
        // Get CEK
        let cek = if let Some(cached) = self.cek_cache.get(&key_uuid.to_string()) {
            cached.clone()
        } else {
             // Find AccessRight
             let right = group.access_rights.iter()
                 .find(|ar| ar.consumer_id == self.consumer_id)
                 .ok_or_else(|| Lib3mfError::Validation(format!("No access right for consumer {} in group {}", self.consumer_id, key_uuid)))?;
                 
             // Unwrap
             // Decode wrapped key from base64 first? 
             // Phase 13 parser might have stored it as base64 string or Vec<u8>?
             // Checking safe_content.rs: wrapped_key: Vec<u8>
             // So it's already bytes (parser handled it? Parser code snippet showed `read_text_content`? 
             // `secure_content_parser.rs` snippet didn't show `WrappedKey` parsing detail in initial view.
             // Assuming it's Vec<u8> in struct.
             // If parser stored base64 bytes, we need to decode?
             // `scan` of `secure_content_parser.rs` in Phase 13 summary mentioned `wrappedkey`.
             // I'll assume it's raw bytes (decoded by parser) OR raw base64 string bytes.
             // Usually parser decodes text content? No, parser `read_text_content` returns String.
             // But `WrappedKey` in struct is `Vec<u8>`. `secure_content_parser` must have decoded it.
             // I'll verify if `secure_content_parser` did base64 decode.
             // If not, unwrap_key will fail.
             // I'll assume parser did its job or key is not base64 encoded in XML (unlikely, usually base64).
             // Wait, KeyStore struct definition says `wrapped_key: Vec<u8>`.
             // I will try to unwrap.
             
             let cek = KeyManager::unwrap_key(&self.private_key, &right.wrapped_key)?;
             self.cek_cache.insert(key_uuid.to_string(), cek.clone());
             cek
        };
        
        // Read encrypted content
        let encrypted_data = archiver.read_entry(path)?;
        
        // Decrypt
        // Format of encrypted data? 
        // 3MF Spec: "The entire part is encrypted... IV? Tag?"
        // Usually IV is prepended.
        // `decrypt_aes256gcm` implementation in `encryption.rs` expects `nonce` + `ciphertext` (where ciphertext includes tag).
        // `aes_gcm` crate `encrypt` outputs `ciphertext` (with tag).
        // My `decrypt_aes256gcm` takes `nonce` and `ciphertext` separately.
        // I need to split the file content.
        // Nonce is 12 bytes.
        // So: Nonce (12) | Ciphertext (rest).
        
        if encrypted_data.len() < 12 + 16 { // Nonce + Tag
             return Err(Lib3mfError::Validation("Encrypted file too short".to_string()));
        }
        
        let (nonce, ciphertext) = encrypted_data.split_at(12);
        
        let plaintext = decrypt_aes256gcm(&cek, nonce, ciphertext)?;
        
        Ok(Some(plaintext))
    }
}
