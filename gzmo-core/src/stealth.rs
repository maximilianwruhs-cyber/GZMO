use std::path::Path;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HardwareFingerprint {
    pub has_nvidia_gpu: bool,
    pub has_amd_gpu: bool,
    pub has_avx2: bool,
}

/// Discovers hardware capabilities using strictly passive userland and virtual file system probes.
/// This prevents EDRs from flagging the daemon for spawning hardware recon subprocesses
/// like `nvidia-smi` or `rocminfo`.
pub fn discover_hardware_stealthily() -> HardwareFingerprint {
    info!("Running passive stealth hardware discovery");

    let mut fingerprint = HardwareFingerprint {
        has_nvidia_gpu: false,
        has_amd_gpu: false,
        has_avx2: false,
    };

    // 1. NVIDIA CUDA Discovery (Linux)
    // Passively stat() virtual folders.
    // We strictly use path.exists() which triggers a simple `stat` syscall instead of `open`.
    if Path::new("/sys/module/nvidia").exists() || Path::new("/dev/nvidiactl").exists() {
        debug!("Discovered NVIDIA GPU ecosystem via passive sysfs/devfs stat");
        fingerprint.has_nvidia_gpu = true;
    }

    // 2. AMD ROCm Discovery (Linux)
    // Passively check for Kernel Fusion Driver endpoint node
    if Path::new("/dev/kfd").exists() {
        debug!("Discovered AMD GPU ecosystem via passive KFD dispatcher stat");
        fingerprint.has_amd_gpu = true;
    }

    // 3. CPU Fallback (Graceful Discovery)
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        debug!("Discovered CPU AVX2 acceleration capabilities natively");
        fingerprint.has_avx2 = true;
    }

    info!(
        "Stealth discovery complete. CUDA: {}, ROCm: {}, AVX2: {}",
        fingerprint.has_nvidia_gpu,
        fingerprint.has_amd_gpu,
        fingerprint.has_avx2
    );

    fingerprint
}
