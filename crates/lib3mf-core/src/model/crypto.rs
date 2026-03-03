use serde::{Deserialize, Serialize};

/// Represents an XML-DSIG Signature element.
/// Namespace: <http://www.w3.org/2000/09/xmldsig#>
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Signature {
    /// Signed content metadata including canonicalization and digest references.
    pub signed_info: SignedInfo,
    /// The base64-encoded cryptographic signature value.
    pub signature_value: SignatureValue,
    /// Key identification information (certificate, key name, or RSA public key).
    pub key_info: Option<KeyInfo>,
    // Object element is not typically used for simple 3MF signatures but spec allows it?
    // We'll stick to core elements first.
}

/// XML-DSIG `SignedInfo` element containing the canonicalization algorithm, signature method, and references.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignedInfo {
    /// The XML canonicalization algorithm URI.
    pub canonicalization_method: CanonicalizationMethod,
    /// The signature algorithm URI (e.g., RSA-SHA256).
    pub signature_method: SignatureMethod,
    /// List of references to signed content with their digest values.
    pub references: Vec<Reference>,
}

/// XML canonicalization method, identified by an algorithm URI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonicalizationMethod {
    /// Algorithm URI (e.g., `"http://www.w3.org/TR/2001/REC-xml-c14n-20010315"`).
    pub algorithm: String,
}

/// Signature algorithm specification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureMethod {
    /// Algorithm URI (e.g., `"http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"`).
    pub algorithm: String,
}

/// A single XML-DSIG reference pointing to a signed resource with its digest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reference {
    /// URI of the referenced resource (e.g., `/3D/3dmodel.model`).
    pub uri: String,
    /// The digest (hash) algorithm used.
    pub digest_method: DigestMethod,
    /// The base64-encoded digest value of the referenced resource.
    pub digest_value: DigestValue,
    // Transforms are optional in 3MF restricted profile (usually C14N is implicit or specified)
    /// Optional list of transforms applied before digesting.
    pub transforms: Option<Vec<Transform>>,
}

/// An XML transform applied to a reference before computing its digest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transform {
    /// Algorithm URI identifying the transform.
    pub algorithm: String,
}

/// Digest algorithm specification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestMethod {
    /// Algorithm URI (e.g., `"http://www.w3.org/2001/04/xmlenc#sha256"`).
    pub algorithm: String,
}

/// Base64-encoded digest value from an XML-DSIG reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestValue {
    // Base64 encoded value usually. We store as simple String or bytes?
    // String for XML mapping, bytes for logic?
    // Let's store raw string here to match XML, decode later.
    /// Base64-encoded digest string as it appears in the XML.
    pub value: String,
}

/// Base64-encoded XML-DSIG signature value.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureValue {
    /// Base64-encoded signature bytes as they appear in the XML.
    pub value: String,
}

/// Key identification information for verifying or locating the signing key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyInfo {
    // For 3MF, usually KeyName (UUID) or KeyValue (RSA Public Key)
    /// Optional key name or UUID identifying the signing key.
    pub key_name: Option<String>,
    /// Optional RSA public key value embedded in the signature.
    pub key_value: Option<KeyValue>,
    /// Optional X.509 certificate data for the signing key.
    pub x509_data: Option<X509Data>,
}

/// X.509 certificate container for signature verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct X509Data {
    /// Base64-encoded PEM or DER certificate bytes.
    pub certificate: Option<String>, // Base64 encoded PEM/DER
}

/// RSA or other asymmetric public key value embedded in a signature.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyValue {
    /// RSA key parameters (modulus and exponent).
    pub rsa_key_value: Option<RSAKeyValue>,
}

/// RSA public key parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RSAKeyValue {
    /// Base64-encoded RSA modulus.
    pub modulus: String,
    /// Base64-encoded RSA public exponent.
    pub exponent: String,
}

// Helper types for Keystore mapping
/// Parsed metadata from an X.509 certificate used for signing.
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// Certificate subject distinguished name.
    pub subject: String,
    /// Certificate issuer distinguished name.
    pub issuer: String,
    /// Certificate serial number.
    pub serial_number: String,
    // Real parsed data could be stored if we hold the X509Certificate object,
    // but usually we just parse on demand from the PEM/DER bytes.
}
