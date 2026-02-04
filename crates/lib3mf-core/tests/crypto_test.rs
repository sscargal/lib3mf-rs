use lib3mf_core::crypto::encryption::{decrypt_aes256gcm, encrypt_aes256gcm};
use lib3mf_core::crypto::keys::KeyManager;
use lib3mf_core::model::KeyStore;
use rand::RngCore;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;

// ============================================================================
// AES-256-GCM Tests
// ============================================================================

#[test]
fn test_aes_gcm_roundtrip_success() {
    // PRIMARY SUCCESS PATH: Encrypt and decrypt 1KB of random data
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let mut data = vec![0u8; 1024];
    rand::thread_rng().fill_bytes(&mut data);

    let (encrypted, nonce) = encrypt_aes256gcm(&key, &data).expect("Encryption should succeed");

    // Verify ciphertext is different from plaintext
    assert_ne!(
        data.as_slice(),
        encrypted.as_slice(),
        "Ciphertext should differ from plaintext"
    );

    // Verify ciphertext is longer (includes authentication tag)
    assert!(
        encrypted.len() >= data.len(),
        "Ciphertext should be at least as long as plaintext"
    );

    let decrypted = decrypt_aes256gcm(&key, &nonce, &encrypted).expect("Decryption should succeed");

    // CRITICAL: Verify decrypted data matches original
    assert_eq!(
        data.as_slice(),
        decrypted.as_slice(),
        "Decrypted data must match original plaintext"
    );
}

#[test]
fn test_aes_gcm_roundtrip() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = b"Top secret 3D model data that needs encryption.";

    let (encrypted, nonce) = encrypt_aes256gcm(&key, data).expect("Encryption failed");
    assert_ne!(
        data.as_ref(),
        encrypted.as_slice(),
        "Ciphertext should differ from plaintext"
    );

    let decrypted = decrypt_aes256gcm(&key, &nonce, &encrypted).expect("Decryption failed");
    assert_eq!(
        data.as_ref(),
        decrypted.as_slice(),
        "Decrypted data should match original"
    );
}

#[test]
fn test_aes_gcm_empty_plaintext() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = b"";

    let (encrypted, nonce) =
        encrypt_aes256gcm(&key, data).expect("Encrypting empty data should succeed");

    // Empty plaintext still produces ciphertext (just the authentication tag)
    assert!(
        !encrypted.is_empty(),
        "Encrypted empty data should still have authentication tag"
    );

    let decrypted =
        decrypt_aes256gcm(&key, &nonce, &encrypted).expect("Decrypting empty data should succeed");
    assert_eq!(
        data.as_ref(),
        decrypted.as_slice(),
        "Decrypted empty data should match"
    );
}

#[test]
fn test_aes_gcm_large_data() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    // 1MB of data
    let mut data = vec![0u8; 1024 * 1024];
    rand::thread_rng().fill_bytes(&mut data);

    let (encrypted, nonce) =
        encrypt_aes256gcm(&key, &data).expect("Encrypting large data should succeed");
    let decrypted =
        decrypt_aes256gcm(&key, &nonce, &encrypted).expect("Decrypting large data should succeed");

    assert_eq!(
        data.as_slice(),
        decrypted.as_slice(),
        "Large data should roundtrip correctly"
    );
}

#[test]
fn test_aes_gcm_wrong_key_fails() {
    let mut key1 = [0u8; 32];
    let mut key2 = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key1);
    rand::thread_rng().fill_bytes(&mut key2);

    let data = b"Encrypted with key1";

    let (encrypted, nonce) = encrypt_aes256gcm(&key1, data).expect("Encryption should succeed");

    // Attempt to decrypt with wrong key
    let result = decrypt_aes256gcm(&key2, &nonce, &encrypted);
    assert!(result.is_err(), "Decryption with wrong key should fail");

    if let Err(e) = result {
        let err_msg = format!("{}", e);
        assert!(
            err_msg.contains("Decryption failed") || err_msg.contains("EncryptionError"),
            "Error should indicate decryption failure: {}",
            err_msg
        );
    }
}

#[test]
fn test_aes_gcm_wrong_nonce_fails() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = b"Encrypted with nonce1";

    let (encrypted, _nonce1) = encrypt_aes256gcm(&key, data).expect("Encryption should succeed");

    // Use different nonce
    let mut wrong_nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut wrong_nonce);

    let result = decrypt_aes256gcm(&key, &wrong_nonce, &encrypted);
    assert!(result.is_err(), "Decryption with wrong nonce should fail");
}

#[test]
fn test_aes_gcm_invalid_key_length() {
    let short_key = [0u8; 16]; // 128-bit key instead of 256-bit
    let data = b"test data";

    let result = encrypt_aes256gcm(&short_key, data);
    assert!(result.is_err(), "Encryption with 16-byte key should fail");

    if let Err(e) = result {
        let err_msg = format!("{}", e);
        assert!(
            err_msg.contains("Invalid key length"),
            "Error should mention invalid key length: {}",
            err_msg
        );
    }

    let long_key = [0u8; 64]; // 512-bit key
    let result = encrypt_aes256gcm(&long_key, data);
    assert!(result.is_err(), "Encryption with 64-byte key should fail");
}

#[test]
fn test_aes_gcm_invalid_nonce_length() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = b"test data";
    let (ciphertext, _) = encrypt_aes256gcm(&key, data).expect("Encryption should succeed");

    // Try with wrong nonce length (8 bytes instead of 12)
    let short_nonce = [0u8; 8];
    let result = decrypt_aes256gcm(&key, &short_nonce, &ciphertext);
    assert!(result.is_err(), "Decryption with 8-byte nonce should fail");

    // Try with wrong nonce length (16 bytes instead of 12)
    let long_nonce = [0u8; 16];
    let result = decrypt_aes256gcm(&key, &long_nonce, &ciphertext);
    assert!(result.is_err(), "Decryption with 16-byte nonce should fail");
}

#[test]
fn test_aes_gcm_tampered_ciphertext() {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = b"Important data that must not be tampered with";

    let (mut encrypted, nonce) = encrypt_aes256gcm(&key, data).expect("Encryption should succeed");

    // Tamper with a byte in the middle of the ciphertext
    if encrypted.len() > 10 {
        let idx = encrypted.len() / 2;
        encrypted[idx] ^= 0xFF;
    }

    let result = decrypt_aes256gcm(&key, &nonce, &encrypted);
    assert!(
        result.is_err(),
        "Decryption of tampered ciphertext should fail (authentication failure)"
    );
}

// ============================================================================
// RSA Key Management Tests
// ============================================================================

fn generate_test_key() -> RsaPrivateKey {
    // Generate a fresh 2048-bit key for testing
    use rand::rngs::OsRng;
    RsaPrivateKey::new(&mut OsRng, 2048).expect("Failed to generate test key")
}

#[test]
fn test_key_wrap_unwrap_roundtrip() {
    let private_key = generate_test_key();
    let public_key = private_key.to_public_key();

    // Generate a random 32-byte CEK (Content Encryption Key)
    let mut cek = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut cek);

    // Wrap the CEK with the public key
    let wrapped = KeyManager::wrap_key(&public_key, &cek).expect("Key wrapping should succeed");

    // Verify wrapped key is different from CEK and is RSA-sized (256 bytes for 2048-bit key)
    assert_ne!(
        wrapped.as_slice(),
        cek.as_ref(),
        "Wrapped key should differ from CEK"
    );
    assert_eq!(
        wrapped.len(),
        256,
        "Wrapped key should be 256 bytes for 2048-bit RSA key"
    );

    // Unwrap the key with the private key
    let unwrapped =
        KeyManager::unwrap_key(&private_key, &wrapped).expect("Key unwrapping should succeed");

    // CRITICAL: Verify unwrapped key matches original CEK
    assert_eq!(
        unwrapped.as_slice(),
        cek.as_ref(),
        "Unwrapped key must match original CEK"
    );
}

#[test]
fn test_wrap_key_too_large() {
    let private_key = generate_test_key();
    let public_key = private_key.to_public_key();

    // Try to wrap a key that's too large (RSA-OAEP has maximum message size)
    // For 2048-bit RSA with SHA-1, max is approximately 214 bytes
    let large_key = vec![0u8; 300];

    let result = KeyManager::wrap_key(&public_key, &large_key);
    assert!(result.is_err(), "Wrapping oversized key should fail");

    if let Err(e) = result {
        let err_msg = format!("{}", e);
        assert!(
            err_msg.contains("Key wrapping failed") || err_msg.contains("EncryptionError"),
            "Error should indicate key wrapping failure: {}",
            err_msg
        );
    }
}

#[test]
fn test_unwrap_key_invalid_ciphertext() {
    let private_key = generate_test_key();

    // Try to unwrap random bytes (not valid RSA ciphertext)
    let mut invalid_wrapped = vec![0u8; 256];
    rand::thread_rng().fill_bytes(&mut invalid_wrapped);

    let result = KeyManager::unwrap_key(&private_key, &invalid_wrapped);
    assert!(result.is_err(), "Unwrapping invalid ciphertext should fail");
}

#[test]
fn test_load_private_key_pkcs1_der() {
    // Generate and encode in DER format
    let private_key = generate_test_key();
    use rsa::pkcs1::EncodeRsaPrivateKey;

    let der = private_key
        .to_pkcs1_der()
        .expect("Failed to encode key to DER");

    // Load back from DER
    let loaded =
        RsaPrivateKey::from_pkcs1_der(der.as_bytes()).expect("Loading PKCS#1 DER should succeed");
    assert_eq!(loaded.size(), 256, "Key should be 2048 bits (256 bytes)");
}

#[test]
fn test_load_private_key_pkcs8_pem() {
    // Generate and encode in PKCS#8 PEM format
    let private_key = generate_test_key();
    use rsa::pkcs8::EncodePrivateKey;

    let pem = private_key
        .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
        .expect("Failed to encode key to PEM");

    // Load back from PEM
    let loaded =
        RsaPrivateKey::from_pkcs8_pem(pem.as_str()).expect("Loading PKCS#8 PEM should succeed");
    assert_eq!(loaded.size(), 256, "Key should be 2048 bits (256 bytes)");
}

// ============================================================================
// KeyStore Structure Tests
// ============================================================================

#[test]
fn test_keystore_creation() {
    let keystore = KeyStore::default();
    assert_eq!(
        keystore.consumers.len(),
        0,
        "New keystore should have no consumers"
    );
    assert_eq!(
        keystore.resource_data_groups.len(),
        0,
        "New keystore should have no resource data groups"
    );
}

#[test]
fn test_keystore_add_consumer() {
    use lib3mf_core::model::Consumer;
    use uuid::Uuid;

    let mut keystore = KeyStore {
        uuid: Uuid::new_v4(),
        consumers: Vec::new(),
        resource_data_groups: Vec::new(),
    };

    keystore.consumers.push(Consumer {
        id: "alice@example.com".to_string(),
        key_id: Some("key-001".to_string()),
        key_value: Some("value-001".to_string()),
    });

    keystore.consumers.push(Consumer {
        id: "bob@example.com".to_string(),
        key_id: None,
        key_value: None,
    });

    assert_eq!(
        keystore.consumers.len(),
        2,
        "Keystore should have 2 consumers"
    );
    assert_eq!(
        keystore.consumers[0].id, "alice@example.com",
        "First consumer ID should match"
    );
    assert_eq!(
        keystore.consumers[1].id, "bob@example.com",
        "Second consumer ID should match"
    );
}

#[test]
fn test_keystore_add_resource_group() {
    use lib3mf_core::model::{AccessRight, ResourceDataGroup};
    use uuid::Uuid;

    let mut keystore = KeyStore {
        uuid: Uuid::new_v4(),
        consumers: Vec::new(),
        resource_data_groups: Vec::new(),
    };

    let group_uuid = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();

    keystore.resource_data_groups.push(ResourceDataGroup {
        key_uuid: group_uuid,
        access_rights: vec![
            AccessRight {
                consumer_id: "alice@example.com".to_string(),
                algorithm: "RSA-OAEP".to_string(),
                wrapped_key: vec![0x01, 0x02, 0x03, 0x04],
            },
            AccessRight {
                consumer_id: "bob@example.com".to_string(),
                algorithm: "RSA-OAEP".to_string(),
                wrapped_key: vec![0x05, 0x06, 0x07, 0x08],
            },
        ],
    });

    assert_eq!(
        keystore.resource_data_groups.len(),
        1,
        "Keystore should have 1 resource data group"
    );
    let group = &keystore.resource_data_groups[0];
    assert_eq!(group.key_uuid, group_uuid, "Group UUID should match");
    assert_eq!(
        group.access_rights.len(),
        2,
        "Group should have 2 access rights"
    );
}

#[test]
fn test_keystore_structure() {
    use lib3mf_core::model::{AccessRight, Consumer, ResourceDataGroup};
    use uuid::Uuid;

    // Test that we can build a complete keystore structure
    let keystore_uuid = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
    let group_uuid = Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap();

    let keystore = KeyStore {
        uuid: keystore_uuid,
        consumers: vec![Consumer {
            id: "test@example.com".to_string(),
            key_id: Some("key-123".to_string()),
            key_value: None,
        }],
        resource_data_groups: vec![ResourceDataGroup {
            key_uuid: group_uuid,
            access_rights: vec![AccessRight {
                consumer_id: "test@example.com".to_string(),
                algorithm: "RSA-OAEP".to_string(),
                wrapped_key: vec![0xDE, 0xAD, 0xBE, 0xEF],
            }],
        }],
    };

    // Verify structure integrity
    assert_eq!(keystore.uuid, keystore_uuid, "Keystore UUID should match");
    assert_eq!(keystore.consumers.len(), 1, "Should have 1 consumer");
    assert_eq!(
        keystore.resource_data_groups.len(),
        1,
        "Should have 1 resource data group"
    );
    assert_eq!(
        keystore.resource_data_groups[0].key_uuid, group_uuid,
        "Group UUID should match"
    );
    assert_eq!(
        keystore.resource_data_groups[0].access_rights[0].consumer_id, "test@example.com",
        "Consumer ID should match"
    );
}
