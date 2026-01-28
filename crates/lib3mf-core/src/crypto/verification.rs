use crate::error::{Lib3mfError, Result};
use crate::model::crypto::*;
use crate::model::crypto::*;
use rsa::RsaPublicKey;
use rsa::signature::{Verifier, SignatureEncoding};
use sha2::{Sha256, Digest};
use sha1::Sha1;
use base64::prelude::*;

/// Verifies a 3MF XML digital signature.
/// 
/// `public_key`: The RSA public key to use for verification.
/// `content_resolver`: A closure that takes a URI and returns the content bytes.
/// `signed_info_bytes`: The RAW canonicalized bytes of the <SignedInfo> element. 
///                      Note: This is critical. The parser must extract the exact bytes used for signing.
///                      3MF usually requires C14N. If we don't have a C14N library, 
///                      we must rely on the raw bytes from the file matching the canonical form 
///                      (which is often true for generated files) or implement minimal C14N.
pub fn verify_signature<F>(
    signature: &Signature, 
    public_key: &RsaPublicKey, 
    content_resolver: F,
    signed_info_bytes: &[u8]
) -> Result<bool> 
where F: Fn(&str) -> Result<Vec<u8>>
{
    // 1. Verify References
    for reference in &signature.signed_info.references {
        verify_reference(reference, &content_resolver)?;
    }

    // 2. Verify SignatureValue
    let sig_value = BASE64_STANDARD.decode(&signature.signature_value.value)
        .map_err(|e| Lib3mfError::Validation(format!("Invalid base64 signature: {}", e)))?;
        
    // 3MF uses RSA-SHA256 usually for SignatureMethod.
    // Check algo.
    match signature.signed_info.signature_method.algorithm.as_str() {
        "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256" => {
             // RSA-SHA256
             let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha256>::new(public_key.clone());
             let rsa_signature = rsa::pkcs1v15::Signature::try_from(sig_value.as_slice())
                .map_err(|e| Lib3mfError::Validation(format!("Invalid RSA signature format: {}", e)))?;
             
             verifying_key.verify(signed_info_bytes, &rsa_signature)
                 .map_err(|e| Lib3mfError::Validation(format!("Signature verification failed: {}", e)))?;
        }
        _ => return Err(Lib3mfError::Validation(format!("Unsupported signature method: {}", signature.signed_info.signature_method.algorithm))),
    }
    
    Ok(true)
}

fn verify_reference<F>(reference: &Reference, content_resolver: &F) -> Result<()> 
where F: Fn(&str) -> Result<Vec<u8>>
{
    // Resolve content
    let content = content_resolver(&reference.uri)?;
    
    // Calculate Digest
    let calculated_digest = match reference.digest_method.algorithm.as_str() {
        "http://www.w3.org/2001/04/xmlenc#sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hasher.finalize().to_vec()
        }
        "http://www.w3.org/2000/09/xmldsig#sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(&content);
            hasher.finalize().to_vec()
        }
        _ => return Err(Lib3mfError::Validation(format!("Unsupported digest method: {}", reference.digest_method.algorithm))),
    };
    
    // Decode stored digest
    let stored_digest = BASE64_STANDARD.decode(&reference.digest_value.value)
         .map_err(|e| Lib3mfError::Validation(format!("Invalid base64 digest: {}", e)))?;
         
    if calculated_digest != stored_digest {
        return Err(Lib3mfError::Validation(format!("Digest mismatch for URI {}", reference.uri)));
    }
    
    Ok(())
}
