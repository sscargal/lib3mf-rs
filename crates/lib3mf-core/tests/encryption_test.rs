use lib3mf_core::crypto::encryption::{encrypt_aes256gcm, decrypt_aes256gcm};

#[test]
fn test_aes_gcm_roundtrip() {
    let key = [0x42u8; 32];
    let plaintext = b"Hello 3MF Secure Content!";
    
    let (ciphertext, nonce) = encrypt_aes256gcm(&key, plaintext).expect("Encryption failed");
    
    assert_ne!(ciphertext, plaintext);
    assert_eq!(nonce.len(), 12);
    
    let decrypted = decrypt_aes256gcm(&key, &nonce, &ciphertext).expect("Decryption failed");
    
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_tamper() {
    let key = [0x42u8; 32];
    let plaintext = b"Sensitive Data";
    
    let (mut ciphertext, nonce) = encrypt_aes256gcm(&key, plaintext).expect("Encryption failed");
    
    // Tamper with last byte
    let len = ciphertext.len();
    ciphertext[len-1] ^= 0xFF;
    
    let res = decrypt_aes256gcm(&key, &nonce, &ciphertext);
    assert!(res.is_err());
}
