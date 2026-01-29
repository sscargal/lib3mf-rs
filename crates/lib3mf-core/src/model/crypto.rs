use serde::{Deserialize, Serialize};

/// Represents an XML-DSIG Signature element.
/// Namespace: http://www.w3.org/2000/09/xmldsig#
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Signature {
    pub signed_info: SignedInfo,
    pub signature_value: SignatureValue,
    pub key_info: Option<KeyInfo>,
    // Object element is not typically used for simple 3MF signatures but spec allows it?
    // We'll stick to core elements first.
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignedInfo {
    pub canonicalization_method: CanonicalizationMethod,
    pub signature_method: SignatureMethod,
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonicalizationMethod {
    pub algorithm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureMethod {
    pub algorithm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reference {
    pub uri: String,
    pub digest_method: DigestMethod,
    pub digest_value: DigestValue,
    // Transforms are optional in 3MF restricted profile (usually C14N is implicit or specified)
    pub transforms: Option<Vec<Transform>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transform {
    pub algorithm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestMethod {
    pub algorithm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DigestValue {
    // Base64 encoded value usually. We store as simple String or bytes?
    // String for XML mapping, bytes for logic?
    // Let's store raw string here to match XML, decode later.
    pub value: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureValue {
    pub value: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyInfo {
    // Could be KeyValue, KeyName, X509Data...
    // For 3MF, usually KeyName (UUID) or KeyValue (RSA Public Key)
    pub key_name: Option<String>,
    pub key_value: Option<KeyValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyValue {
    pub rsa_key_value: Option<RSAKeyValue>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RSAKeyValue {
    pub modulus: String,
    pub exponent: String,
}
