use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce
};
use crate::error::{Lib3mfError, Result};
use rand::RngCore;

/// Encrypts data using AES-256-GCM.
/// Returns (ciphertext, nonce, tag) combined or separate?
/// 3MF Spec: Content Encryption usually has IV/Nonce separately or prepended.
/// "The Key used is the Content Encryption Key (CEK)."
/// "The Algorithm is AES-256-GCM."
/// Nonce is 12 bytes (96 bits).
/// Tag is 16 bytes (128 bits).
/// Output usually: Nonce + Ciphertext (which includes Tag usually in Rust implementation?)
/// Rust `aes-gcm` crate returns `ciphertext + tag`.
pub fn encrypt_aes256gcm(key: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    if key.len() != 32 {
        return Err(Lib3mfError::Validation("Invalid key length for AES-256-GCM".to_string()));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    
    // Generate random 96-bit nonce
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| Lib3mfError::EncryptionError(format!("Encryption failed: {}", e)))?;
        
    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypts data using AES-256-GCM.
/// Input: ciphertext (including tag), key, nonce.
pub fn decrypt_aes256gcm(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
     if key.len() != 32 {
        return Err(Lib3mfError::Validation("Invalid key length for AES-256-GCM".to_string()));
    }
    if nonce.len() != 12 {
        return Err(Lib3mfError::Validation("Invalid nonce length for AES-256-GCM".to_string()));
    }
    
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
         .map_err(|e| Lib3mfError::EncryptionError(format!("Decryption failed: {}", e)))?;
         
    Ok(plaintext)
}
