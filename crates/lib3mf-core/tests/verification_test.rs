use lib3mf_core::crypto::verification::verify_signature;
use lib3mf_core::model::crypto::*;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Sign};
use rsa::signature::{Signer, SignatureEncoding};
use rsa::pkcs1::EncodeRsaPublicKey;
use sha2::{Sha256, Digest};
use base64::prelude::*;

#[test]
fn test_signature_verification() {
    // 1. Setup Keys
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
    let pub_key = RsaPublicKey::from(&priv_key);
    
    // 2. Mock Content
    let content_uri = "/3D/model.model";
    let content_bytes = b"Actual 3D Model Content";
    let content_resolver = |uri: &str| -> Result<Vec<u8>, lib3mf_core::error::Lib3mfError> {
        if uri == content_uri {
            Ok(content_bytes.to_vec())
        } else {
            Err(lib3mf_core::error::Lib3mfError::ResourceNotFound(0))
        }
    };
    
    // 3. Create Signature Structure
    let mut signature = Signature::default();
    
    // Reference
    let mut reference = Reference::default();
    reference.uri = content_uri.to_string();
    reference.digest_method.algorithm = "http://www.w3.org/2001/04/xmlenc#sha256".to_string();
    
    // Calculate digest
    let mut hasher = Sha256::new();
    hasher.update(content_bytes);
    let digest = hasher.finalize();
    reference.digest_value.value = BASE64_STANDARD.encode(digest);
    
    signature.signed_info.references.push(reference);
    
    signature.signed_info.canonicalization_method.algorithm = "http://www.w3.org/TR/2001/REC-xml-c14n-20010315".to_string();
    signature.signed_info.signature_method.algorithm = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256".to_string();
    
    // 4. Sign SignedInfo
    // In real world, we would canonicalize the <SignedInfo> Element.
    // Here we just use a mock byte representation of it.
    let signed_info_bytes = b"<SignedInfo>Mock Canonicalized Content</SignedInfo>";
    
    let signing_key = rsa::pkcs1v15::SigningKey::<Sha256>::new(priv_key);
    let rsa_signature = signing_key.sign(signed_info_bytes); // Correct: use sign which handles hashing internally for Pkcs1v15Sign?
    // Wait, Pkcs1v15Sign takes a Digest usually? Or `sign` method?
    // rsa::pkcs1v15::SigningKey implements RandomizedSigner (or Signer depending on version).
    // Signer trait has `sign`.
    // The signature should be over the bytes.
    
    signature.signature_value.value = BASE64_STANDARD.encode(rsa_signature.to_bytes());
    
    // 5. Verify
    let result = verify_signature(&signature, &pub_key, content_resolver, signed_info_bytes);
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_signature_tamper_content() {
     // 1. Setup Keys
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
    let pub_key = RsaPublicKey::from(&priv_key);
    
    // 2. Mock Content
    let content_uri = "/3D/model.model";
    let content_bytes = b"Actual 3D Model Content";
    let content_resolver = |uri: &str| -> Result<Vec<u8>, lib3mf_core::error::Lib3mfError> {
        if uri == content_uri {
             // Return TAMPERED content
            Ok(b"Tampered Content".to_vec())
        } else {
            Err(lib3mf_core::error::Lib3mfError::ResourceNotFound(0))
        }
    };
    
    // 3. Create Signature Structure (Same as valid one)
    let mut signature = Signature::default();
    
    let mut reference = Reference::default();
    reference.uri = content_uri.to_string();
    reference.digest_method.algorithm = "http://www.w3.org/2001/04/xmlenc#sha256".to_string();
    
    let mut hasher = Sha256::new();
    hasher.update(content_bytes); // Hash of ORIGINAL content
    let digest = hasher.finalize();
    reference.digest_value.value = BASE64_STANDARD.encode(digest);
    
    signature.signed_info.references.push(reference);
    signature.signed_info.signature_method.algorithm = "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256".to_string();
    
    let signed_info_bytes = b"<SignedInfo>Mock Canonicalized Content</SignedInfo>";
    let signing_key = rsa::pkcs1v15::SigningKey::<Sha256>::new(priv_key);
    let rsa_signature = signing_key.sign(signed_info_bytes);
    signature.signature_value.value = BASE64_STANDARD.encode(rsa_signature.to_bytes());

    // 5. Verify - Should Fail Digest Check
    let result = verify_signature(&signature, &pub_key, content_resolver, signed_info_bytes);
    
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("Digest mismatch"));
}
