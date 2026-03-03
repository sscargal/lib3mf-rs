use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the Secure Content KeyStore, managing keys and access rights.
/// In 3MF, this holds info about Consumers (recipients) and which resources they can decrypt.
/// Typical flow: Resource is encrypted -> ResourceDataGroup.
/// ResourceDataGroup key is wrapped for each Consumer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyStore {
    /// Unique identifier for this key store.
    pub uuid: Uuid,
    /// List of authorized consumers (recipients) who can decrypt resources.
    pub consumers: Vec<Consumer>,
    /// List of resource data groups, each protecting one or more encrypted resources.
    pub resource_data_groups: Vec<ResourceDataGroup>,
}

/// An authorized recipient who can decrypt protected resources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Consumer {
    /// Consumer ID (e.g., email address or a UUID string).
    pub id: String, // Consumer ID (e.g. email or unique string)
    /// Key ID used to wrap (encrypt) the content key for this consumer.
    pub key_id: Option<String>, // Key ID used to wrap the content key
    /// Wrapped (encrypted) content key value, if applicable.
    pub key_value: Option<String>, // Wrapped Key Value usage (if applicable)
                                   // Detailed spec has more fields for X.509 certificates etc.
                                   // For now, we store basic identifiers.
}

/// A group of encrypted resources sharing a single content encryption key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceDataGroup {
    /// UUID of the content encryption key (CEK) protecting this group's resources.
    pub key_uuid: Uuid, // UUID of the content encryption key
    /// Per-consumer access rights specifying how each consumer's wrapped key is provided.
    pub access_rights: Vec<AccessRight>,
    // This group logically contains resources. The resources themselves (Objects, Textures)
    // refer to this group or are implicitly part of it via relationships.
}

/// Per-consumer access right specifying the wrapped content key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessRight {
    /// ID of the consumer this access right is for.
    pub consumer_id: String,
    /// Key wrapping algorithm URI (e.g., RSA-OAEP).
    pub algorithm: String, // Parsing algorithm (e.g. RSA-OAEP)
    /// The content encryption key encrypted for this consumer.
    pub wrapped_key: Vec<u8>, // The encrypted content key for this consumer
}

// Note: In 3MF Secure Content, the actual resources are encrypted in the OPC (ZIP) container.
// The XML metadata describes HOW to decrypt them.
