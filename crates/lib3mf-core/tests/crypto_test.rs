use lib3mf_core::model::KeyStore;
use lib3mf_core::crypto::encryption::{encrypt_aes256gcm, decrypt_aes256gcm};
use rand::RngCore;

#[test]
fn test_aes_gcm_roundtrip() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    
    let data = b"Top secret 3D model data that needs encryption.";
    
    let (encrypted, nonce) = encrypt_aes256gcm(&key, data).expect("Encryption failed");
    assert_ne!(data.as_ref(), encrypted.as_slice());
    
    let decrypted = decrypt_aes256gcm(&key, &nonce, &encrypted).expect("Decryption failed");
    assert_eq!(data.as_ref(), decrypted.as_slice());
}

#[test]
fn test_keystore_creation() {
    let keystore = KeyStore::default();
    assert_eq!(keystore.consumers.len(), 0);
    assert_eq!(keystore.resource_data_groups.len(), 0);
}
