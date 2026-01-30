use crate::error::{Lib3mfError, Result};
use crate::model::crypto::*;
use base64::prelude::*;
use rsa::RsaPublicKey;
use rsa::signature::Verifier;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use x509_parser::prelude::FromDer;

/// Verifies a 3MF XML digital signature.
///
/// `public_key`: The RSA public key to use for verification.
/// `content_resolver`: A closure that takes a URI and returns the content bytes.
/// `signed_info_bytes`: The RAW canonicalized bytes of the `<SignedInfo>` element.
///                      Note: This is critical. The parser must extract the exact bytes used for signing.
///                      3MF usually requires C14N. If we don't have a C14N library,
///                      we must rely on the raw bytes from the file matching the canonical form
///                      (which is often true for generated files) or implement minimal C14N.
pub fn verify_signature<F>(
    signature: &Signature,
    public_key: &RsaPublicKey,
    content_resolver: F,
    signed_info_bytes: &[u8],
) -> Result<bool>
where
    F: Fn(&str) -> Result<Vec<u8>>,
{
    // 1. Verify References
    for reference in &signature.signed_info.references {
        verify_reference(reference, &content_resolver)?;
    }

    // 2. Verify SignatureValue
    let sig_value = BASE64_STANDARD
        .decode(&signature.signature_value.value)
        .map_err(|e| Lib3mfError::Validation(format!("Invalid base64 signature: {}", e)))?;

    // 3MF uses RSA-SHA256 usually for SignatureMethod.
    // Check algo.
    match signature.signed_info.signature_method.algorithm.as_str() {
        "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256" => {
            // RSA-SHA256
            let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha256>::new(public_key.clone());
            let rsa_signature =
                rsa::pkcs1v15::Signature::try_from(sig_value.as_slice()).map_err(|e| {
                    Lib3mfError::Validation(format!("Invalid RSA signature format: {}", e))
                })?;

            verifying_key
                .verify(signed_info_bytes, &rsa_signature)
                .map_err(|e| {
                    Lib3mfError::Validation(format!("Signature verification failed: {}", e))
                })?;
        }
        _ => {
            return Err(Lib3mfError::Validation(format!(
                "Unsupported signature method: {}",
                signature.signed_info.signature_method.algorithm
            )));
        }
    }

    Ok(true)
}

fn verify_reference<F>(reference: &Reference, content_resolver: &F) -> Result<()>
where
    F: Fn(&str) -> Result<Vec<u8>>,
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
        _ => {
            return Err(Lib3mfError::Validation(format!(
                "Unsupported digest method: {}",
                reference.digest_method.algorithm
            )));
        }
    };

    // Decode stored digest
    let stored_digest = BASE64_STANDARD
        .decode(&reference.digest_value.value)
        .map_err(|e| Lib3mfError::Validation(format!("Invalid base64 digest: {}", e)))?;

    if calculated_digest != stored_digest {
        return Err(Lib3mfError::Validation(format!(
            "Digest mismatch for URI {}",
            reference.uri
        )));
    }

    Ok(())
}

/// Verify signature by extracting key from KeyInfo if possible.
pub fn verify_signature_extended<F>(
    signature: &Signature,
    content_resolver: F,
    signed_info_bytes: &[u8],
) -> Result<bool>
where
    F: Fn(&str) -> Result<Vec<u8>>,
{
    let key = extract_key_from_signature(signature)?;
    verify_signature(signature, &key, content_resolver, signed_info_bytes)
}

pub fn extract_key_from_signature(signature: &Signature) -> Result<RsaPublicKey> {
    if let Some(info) = &signature.key_info {
        // 1. Try KeyValue (RSA)
        if let Some(kv) = &info.key_value
            && let Some(rsa_val) = &kv.rsa_key_value
        {
            let n_bytes = BASE64_STANDARD
                .decode(&rsa_val.modulus)
                .map_err(|e| Lib3mfError::Validation(format!("Invalid modulus base64: {}", e)))?;
            let e_bytes = BASE64_STANDARD
                .decode(&rsa_val.exponent)
                .map_err(|e| Lib3mfError::Validation(format!("Invalid exponent base64: {}", e)))?;

            let n = rsa::BigUint::from_bytes_be(&n_bytes);
            let e = rsa::BigUint::from_bytes_be(&e_bytes);

            return RsaPublicKey::new(n, e).map_err(|e| {
                Lib3mfError::Validation(format!("Invalid RSA key components: {}", e))
            });
        }

        // 2. Try X509Data
        if let Some(x509) = &info.x509_data
            && let Some(cert_b64) = &x509.certificate
        {
            // Remove potential headers/whitespace
            let clean_b64: String = cert_b64.chars().filter(|c| !c.is_whitespace()).collect();
            let cert_der = BASE64_STANDARD
                .decode(&clean_b64)
                .map_err(|e| Lib3mfError::Validation(format!("Invalid X509 base64: {}", e)))?;

            let (_, cert) = x509_parser::certificate::X509Certificate::from_der(&cert_der)
                .map_err(|e| Lib3mfError::Validation(format!("Invalid X509 certificate: {}", e)))?;

            // Extract SPKI
            // rsa crate can parse PKCS#1 or PKCS#8. SPKI is usually PKCS#8 compatible (public key info).
            // cert.tbs_certificate.subject_pki contains the SPKI
            use rsa::pkcs8::DecodePublicKey;
            return RsaPublicKey::from_public_key_der(cert.tbs_certificate.subject_pki.raw)
                .map_err(|e| Lib3mfError::Validation(format!("Invalid RSA key in cert: {}", e)));
        }
    }
    Err(Lib3mfError::Validation(
        "No usable KeyValue or X509Certificate found in KeyInfo".into(),
    ))
}
