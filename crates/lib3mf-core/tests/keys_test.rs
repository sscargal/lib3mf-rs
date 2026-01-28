use lib3mf_core::crypto::keys::KeyManager;
use rsa::{RsaPrivateKey, RsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey};
use rand::rngs::OsRng;
// use rand::RngCore;

#[test]
fn test_wrap_unwrap_key() {
    // Generate temporary keys for testing
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
    let pub_key = RsaPublicKey::from(&priv_key);
    
    // CEK (32 bytes for AES-256)
    let cek = [0x55u8; 32];
    
    // Wrap
    let wrapped = KeyManager::wrap_key(&pub_key, &cek).expect("Wrapping failed");
    
    assert_ne!(wrapped, cek);
    // RSA-2048 wrapper size should be 256 bytes (2048 bits)
    assert_eq!(wrapped.len(), 256);
    
    // Unwrap
    let unwrapped = KeyManager::unwrap_key(&priv_key, &wrapped).expect("Unwrapping failed");
    
    assert_eq!(unwrapped, cek);
}
