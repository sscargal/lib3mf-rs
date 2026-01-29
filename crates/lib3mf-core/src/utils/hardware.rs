use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub architecture: String,
    pub num_cpus: usize,
    pub simd_features: Vec<String>,
}

pub fn detect_capabilities() -> HardwareCapabilities {
    let mut features = Vec::new();

    #[cfg(target_arch = "x86_64")]
    {
        if std::is_x86_feature_detected!("sse") { features.push("sse".to_string()); }
        if std::is_x86_feature_detected!("sse2") { features.push("sse2".to_string()); }
        if std::is_x86_feature_detected!("sse3") { features.push("sse3".to_string()); }
        if std::is_x86_feature_detected!("sse4.1") { features.push("sse4.1".to_string()); }
        if std::is_x86_feature_detected!("sse4.2") { features.push("sse4.2".to_string()); }
        if std::is_x86_feature_detected!("avx") { features.push("avx".to_string()); }
        if std::is_x86_feature_detected!("avx2") { features.push("avx2".to_string()); }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // On AArch64 neon is usually always present, but we can check specifically if needed
        // using the cpufeatures crate or other methods.
        features.push("neon".to_string());
    }

    HardwareCapabilities {
        architecture: std::env::consts::ARCH.to_string(),
        num_cpus: rayon::current_num_threads(),
        simd_features: features,
    }
}
