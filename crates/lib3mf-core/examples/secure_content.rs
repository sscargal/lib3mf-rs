#[cfg(feature = "crypto")]
use base64::prelude::*;
#[cfg(feature = "crypto")]
use lib3mf_core::model::crypto::{
    CanonicalizationMethod, DigestMethod, DigestValue, KeyInfo, KeyValue, RSAKeyValue, Reference,
    Signature, SignatureMethod, SignatureValue, SignedInfo,
};
#[cfg(feature = "crypto")]
use rsa::RsaPublicKey;

#[cfg(feature = "crypto")]
fn main() -> anyhow::Result<()> {
    println!("--- 3MF Secure Content Example ---");

    // 1. Mock a Public Key (RSA 2048)
    // In a real app, this comes from a Certificate or KeyStore
    let (n, e) = get_mock_rsa_public_key();
    let n_bytes = n.to_bytes_be();
    let e_bytes = e.to_bytes_be();

    // Construct RsaPublicKey
    let _public_key = RsaPublicKey::new(n.clone(), e.clone())
        .map_err(|e| anyhow::anyhow!("Invalid key: {}", e))?;

    println!("Public Key loaded.");

    // 2. Mock a Signature Structure
    // This represents what we would parse from /Metadata/signature.xml
    let signature = Signature {
        signed_info: SignedInfo {
            canonicalization_method: CanonicalizationMethod {
                algorithm: "http://www.w3.org/TR/2001/REC-xml-c14n-20010315".to_string(),
            },
            signature_method: SignatureMethod {
                algorithm: "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256".to_string(),
            },
            references: vec![Reference {
                uri: "/3D/3dmodel.model".to_string(),
                digest_method: DigestMethod {
                    algorithm: "http://www.w3.org/2001/04/xmlenc#sha256".to_string(),
                },
                digest_value: DigestValue {
                    // SHA256 of "mock model content"
                    value: "K7gNU3lca+9op6oy1rxkTZqH9M99U2B3s+f9Lz/l/Pc=".to_string(),
                },
                transforms: None,
            }],
        },
        signature_value: SignatureValue {
            // Mock signature value (would need real RSA sig to pass)
            value: "MockBase64SignatureValue==".to_string(),
        },
        key_info: Some(KeyInfo {
            key_name: Some("3606f363-d34e-4177-a36c-246e4318c50e".to_string()),
            key_value: Some(KeyValue {
                rsa_key_value: Some(RSAKeyValue {
                    modulus: BASE64_STANDARD.encode(n_bytes),
                    exponent: BASE64_STANDARD.encode(e_bytes),
                }),
            }),
            x509_data: None,
        }),
    };

    println!("Signature structure mock created.");

    // 3. Define a mock content resolver
    // In reality, this reads from the ZipArchiver
    let resolver = |uri: &str| -> lib3mf_core::error::Result<Vec<u8>> {
        if uri == "/3D/3dmodel.model" {
            Ok(b"mock model content".to_vec())
        } else {
            Err(lib3mf_core::error::Lib3mfError::Validation(
                "Not found".to_string(),
            ))
        }
    };

    // 4. Verify (This will specificall fail signature check because mock sig is invalid, but demonstrates flow)
    println!("Verifying signature...");

    // We need canonical bytes of SignedInfo.
    // Since we mocked the struct, we don't have the original bytes.
    // For this example, we'll verify references only if we called a lower level function,
    // but verify_signature checks everything.
    // We expect it to fail on the RSA check.

    // To make it runnable without erroring, we can't truly verify RSA without valid signature.
    // So we'll valid references manually to show success.

    for ref_item in &signature.signed_info.references {
        verify_reference(ref_item, &resolver)?;
        println!("Reference {} Verified!", ref_item.uri);
    }

    println!("(Skipping RSA check as we don't have a valid signature for the mock key)");

    Ok(())
}

#[cfg(not(feature = "crypto"))]
fn main() {
    eprintln!("This example requires the 'crypto' feature to be enabled.");
    eprintln!("Run with: cargo run --example secure_content --features crypto");
}

#[cfg(feature = "crypto")]
fn verify_reference<F>(reference: &Reference, resolver: &F) -> anyhow::Result<()>
where
    F: Fn(&str) -> lib3mf_core::error::Result<Vec<u8>>,
{
    let content = resolver(&reference.uri)?;

    // Hash
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let digest = hasher.finalize();

    // Decode expected
    let expected = BASE64_STANDARD.decode(&reference.digest_value.value)?;

    if digest.as_slice() == expected.as_slice() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Digest mismatch"))
    }
}

#[cfg(feature = "crypto")]
// Deterministic mock key
fn get_mock_rsa_public_key() -> (rsa::BigUint, rsa::BigUint) {
    // 2048-bit modulus (fake small one for example speed/code size)
    // Actually rsa crate requires valid keys.
    // We'll use a hardcoded small key for demo if needed, or just random.
    // Generating a key is slow in debug.
    use rsa::{RsaPrivateKey, traits::PublicKeyParts};
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
    (priv_key.n().clone(), priv_key.e().clone())
}
