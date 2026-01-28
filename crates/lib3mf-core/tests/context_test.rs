use lib3mf_core::crypto::context::SecureContext;
use lib3mf_core::model::secure_content::*;
use lib3mf_core::crypto::keys::KeyManager;
use lib3mf_core::crypto::encryption::encrypt_aes256gcm;
use lib3mf_core::archive::ArchiveReader;
use lib3mf_core::error::Lib3mfError;
use rsa::{RsaPrivateKey, RsaPublicKey};
use uuid::Uuid;
use std::collections::HashMap;

use std::io::{Read, Seek, SeekFrom};

// Mock ArchiveReader
struct MockArchive {
    content: HashMap<String, Vec<u8>>,
}

impl Read for MockArchive {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> { Ok(0) }
}
impl Seek for MockArchive {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> { Ok(0) }
}

impl ArchiveReader for MockArchive {
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, Lib3mfError> {
        self.content.get(name).cloned().ok_or(Lib3mfError::ResourceNotFound(0))
    }
    fn entry_exists(&mut self, name: &str) -> bool { self.content.contains_key(name) }
    fn list_entries(&mut self) -> Result<Vec<String>, Lib3mfError> {
        Ok(self.content.keys().cloned().collect())
    }
}

// Stub implementation for other methods if possible
impl MockArchive {
    fn new() -> Self { Self { content: HashMap::new() } }
}

#[test]
fn test_secure_context_decryption() {
    // 1. Setup Keys
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
    let pub_key = RsaPublicKey::from(&priv_key);
    
    // 2. Setup CEK and Encrypt Content
    let cek = [0x42u8; 32];
    let plaintext_content = b"Super Secret 3D Model Data";
    // Nonce handling inside encrypt_aes256gcm?
    // It returns (ciphertext, nonce).
    // File format: Nonce + Ciphertext
    let (ciphertext_only, nonce) = encrypt_aes256gcm(&cek, plaintext_content).unwrap();
    let mut encrypted_file = nonce;
    encrypted_file.extend(ciphertext_only);
    
    let path = "/3D/encrypted.model";
    
    // 3. Setup Archive
    let mut archiver = MockArchive::new();
    archiver.content.insert(path.to_string(), encrypted_file);
    
    // 4. Setup KeyStore
    let key_uuid = Uuid::new_v4();
    let consumer_id = "test_user";
    
    // Wrap CEK
    let wrapped_key = KeyManager::wrap_key(&pub_key, &cek).unwrap();
    
    let access_right = AccessRight {
        consumer_id: consumer_id.to_string(),
        algorithm: "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p".to_string(),
        wrapped_key,
    };
    
    let group = ResourceDataGroup {
        key_uuid,
        access_rights: vec![access_right],
    };
    
    let keystore = KeyStore {
        uuid: Uuid::new_v4(),
        consumers: vec![Consumer { id: consumer_id.to_string(), key_id: None, key_value: None }],
        resource_data_groups: vec![group],
    };
    
    // 5. Setup SecureContext
    let mut resource_map = HashMap::new();
    resource_map.insert(path.to_string(), key_uuid);
    
    let mut context = SecureContext::new(keystore, priv_key, resource_map, consumer_id.to_string());
    
    // 6. Decrypt
    let result = context.decrypt_entry(&mut archiver, path);
    assert!(result.is_ok());
    let decrypted = result.unwrap().expect("Should return some data");
    
    assert_eq!(decrypted, plaintext_content);
}
