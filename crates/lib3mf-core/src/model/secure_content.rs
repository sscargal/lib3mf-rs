use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the Secure Content KeyStore, managing keys and access rights.
/// In 3MF, this holds info about Consumers (recipients) and which resources they can decrypt.
/// Typical flow: Resource is encrypted -> ResourceDataGroup.
/// ResourceDataGroup key is wrapped for each Consumer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyStore {
    pub uuid: Uuid,
    pub consumers: Vec<Consumer>,
    pub resource_data_groups: Vec<ResourceDataGroup>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Consumer {
    pub id: String,             // Consumer ID (e.g. email or unique string)
    pub key_id: Option<String>, // Key ID used to wrap the content key
    pub key_value: Option<String>, // Wrapped Key Value usage (if applicable)
                                // Detailed spec has more fields for X.509 certificates etc.
                                // For now, we store basic identifiers.
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceDataGroup {
    pub key_uuid: Uuid, // UUID of the content encryption key
    pub access_rights: Vec<AccessRight>,
    // This group logically contains resources. The resources themselves (Objects, Textures)
    // refer to this group or are implicitly part of it via relationships.
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessRight {
    pub consumer_id: String,
    pub algorithm: String,    // Parsing algorithm (e.g. RSA-OAEP)
    pub wrapped_key: Vec<u8>, // The encrypted content key for this consumer
}

// Note: In 3MF Secure Content, the actual resources are encrypted in the OPC (ZIP) container.
// The XML metadata describes HOW to decrypt them.
