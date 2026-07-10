# GZMO Portability Refactoring Guide

**Version:** 1.0  
**Date:** 2026-07-09  
**Priority:** Critical — blocks sharing and deployment

---

## Executive Summary

GZMO currently has **152 machine-specific hardcoded references** that prevent it from running on any other machine without manual code changes. This guide provides a complete refactoring plan to make GZMO truly portable.

### Current State

| Category | Occurrences | Impact |
|----------|-------------|--------|
| Absolute Paths | 11 | Cannot run on different user/home |
| Network Config | 52 | Hardcoded IPs, ports, hostnames |
| Hardware Detection | 11 | Assumes specific GPU/CPU |
| Service Names | 8 | Hardcoded to your infrastructure |
| Directory Structure | 70 | Assumes specific layout |

### Target State

- **0 hardcoded paths** — all paths configurable
- **0 hardcoded IPs** — auto-discovery or config-driven
- **Dynamic hardware detection** — works with any GPU/CPU
- **Abstract service names** — configurable service endpoints
- **Flexible directory structure** — configurable base directories

---

## Architecture: Configuration-Driven Design

### Current Problem

```rust
// HARD-CODED (bad)
const VAULT_PATH: &str = "/home/gzmo/github-clone/GZMO/data/vault.db";
const MODELS_DIR: &str = "/home/gzmo/models/";
const QDRANT_URL: &str = "http://192.168.31.202:6333";
```

### Target Design

```rust
// CONFIGURABLE (good)
#[derive(Debug, Clone)]
pub struct GzmoConfig {
    pub base_dir: PathBuf,
    pub vault_path: PathBuf,
    pub models_dir: PathBuf,
    pub services: ServiceEndpoints,
}

#[derive(Debug, Clone)]
pub struct ServiceEndpoints {
    pub qdrant: String,
    pub ollama: String,
    pub embed_vm: String,
}
```

---

## Refactoring Plan

### Phase 1: Path Abstraction (2-3 hours)

**Goal:** Remove all hardcoded absolute paths

#### 1.1 Create Configuration System

**File:** `gzmo-core/src/config.rs`

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GzmoConfig {
    /// Base directory for all GZMO data
    #[serde(default = "default_base_dir")]
    pub base_dir: PathBuf,
    
    /// Relative paths (resolved against base_dir)
    pub data: DataPaths,
    pub memory: MemoryPaths,
    pub wiki: WikiPaths,
    pub skills: SkillsPaths,
    pub docs: DocsPaths,
    
    /// Service endpoints
    pub services: ServiceEndpoints,
    
    /// Hardware configuration
    pub hardware: HardwareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPaths {
    #[serde(default = "default_vault_path")]
    pub vault: PathBuf,
    #[serde(default = "default_lore_path")]
    pub lore: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPaths {
    #[serde(default = "default_memory_dir")]
    pub episodic: PathBuf,
    #[serde(default = "default_dreams_path")]
    pub dreams: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiPaths {
    #[serde(default = "default_wiki_dir")]
    pub directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsPaths {
    #[serde(default = "default_skills_dir")]
    pub directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsPaths {
    #[serde(default = "default_docs_dir")]
    pub directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    #[serde(default = "default_qdrant_url")]
    pub qdrant: String,
    
    #[serde(default = "default_ollama_url")]
    pub ollama: String,
    
    #[serde(default = "default_embed_vm_url")]
    pub embed_vm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    #[serde(default = "default_gpu_memory_gb")]
    pub gpu_memory_gb: u32,
    
    #[serde(default = "default_cpu_cores")]
    pub cpu_cores: u32,
    
    #[serde(default = "default_ram_gb")]
    pub ram_gb: u32,
}

// Default value functions
fn default_base_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn default_vault_path() -> PathBuf {
    PathBuf::from("data/vault.db")
}

fn default_memory_dir() -> PathBuf {
    PathBuf::from("memory")
}

fn default_wiki_dir() -> PathBuf {
    PathBuf::from("wiki")
}

fn default_skills_dir() -> PathBuf {
    PathBuf::from("skills")
}

fn default_docs_dir() -> PathBuf {
    PathBuf::from("docs")
}

fn default_qdrant_url() -> String {
    "http://localhost:6333".to_string()
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_embed_vm_url() -> String {
    "http://localhost:8081".to_string()
}

fn default_gpu_memory_gb() -> u32 {
    16
}

fn default_cpu_cores() -> u32 {
    8
}

fn default_ram_gb() -> u32 {
    32
}

impl GzmoConfig {
    /// Load configuration from file or create default
    pub fn load_or_create(config_path: &Path) -> Result<Self> {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: GzmoConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = GzmoConfig::default();
            config.save(config_path)?;
            Ok(config)
        }
    }
    
    /// Save configuration to file
    pub fn save(&self, config_path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    /// Resolve a relative path against base_dir
    pub fn resolve_path(&self, relative: &Path) -> PathBuf {
        self.base_dir.join(relative)
    }
    
    /// Get the full path to the vault database
    pub fn vault_path(&self) -> PathBuf {
        self.resolve_path(&self.data.vault)
    }
    
    /// Get the full path to the episodic memory directory
    pub fn memory_dir(&self) -> PathBuf {
        self.resolve_path(&self.memory.episodic)
    }
    
    /// Get the full path to the wiki directory
    pub fn wiki_dir(&self) -> PathBuf {
        self.resolve_path(&self.wiki.directory)
    }
    
    /// Get the full path to the skills directory
    pub fn skills_dir(&self) -> PathBuf {
        self.resolve_path(&self.skills.directory)
    }
    
    /// Get the full path to the docs directory
    pub fn docs_dir(&self) -> PathBuf {
        self.resolve_path(&self.docs.directory)
    }
}

impl Default for GzmoConfig {
    fn default() -> Self {
        Self {
            base_dir: default_base_dir(),
            data: DataPaths {
                vault: default_vault_path(),
                lore: PathBuf::from("data/lore.toml"),
            },
            memory: MemoryPaths {
                episodic: default_memory_dir(),
                dreams: PathBuf::from("DREAMS.md"),
            },
            wiki: WikiPaths {
                directory: default_wiki_dir(),
            },
            skills: SkillsPaths {
                directory: default_skills_dir(),
            },
            docs: DocsPaths {
                directory: default_docs_dir(),
            },
            services: ServiceEndpoints {
                qdrant: default_qdrant_url(),
                ollama: default_ollama_url(),
                embed_vm: default_embed_vm_url(),
            },
            hardware: HardwareConfig {
                gpu_memory_gb: default_gpu_memory_gb(),
                cpu_cores: default_cpu_cores(),
                ram_gb: default_ram_gb(),
            },
        }
    }
}
```

#### 1.2 Update All Path References

**Files to update:**
- `gzmo-core/src/watcher.rs` (line 266)
- `gzmo-cli/src/init_cmd.rs` (lines 197-203)
- `gzmo-cli/src/main.rs` (line 50)
- All other files with hardcoded paths

**Pattern:**
```rust
// BEFORE (bad)
let vault_path = "/home/gzmo/github-clone/GZMO/data/vault.db";
let memory_dir = "/home/gzmo/github-clone/GZMO/memory";

// AFTER (good)
let vault_path = config.vault_path();
let memory_dir = config.memory_dir();
```

#### 1.3 Update Configuration Loading

**File:** `gzmo-cli/src/main.rs`

```rust
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "gzmo")]
#[command(about = "Sovereign Autonomous Agent")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "gzmo.toml")]
    config: PathBuf,
    
    /// Override base directory
    #[arg(short, long)]
    base_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration
    let mut config = GzmoConfig::load_or_create(&cli.config)?;
    
    // Override base_dir if specified
    if let Some(base) = cli.base_dir {
        config.base_dir = base;
    }
    
    // Initialize subsystems with config
    let vault = SqliteVault::new(config.vault_path())?;
    let episodic = FileEpisodicStore::new(config.memory_dir())?;
    let wiki = WikiEngine::new(config.wiki_dir())?;
    
    // ... rest of initialization
}
```

---

### Phase 2: Network Configuration (3-4 hours)

**Goal:** Remove all hardcoded IPs and ports

#### 2.1 Create Service Discovery

**File:** `gzmo-core/src/service_discovery.rs`

```rust
use std::net::TcpStream;
use std::time::Duration;

/// Discover services on the local network
pub struct ServiceDiscovery {
    subnet: String,
    timeout_ms: u64,
}

impl ServiceDiscovery {
    pub fn new(subnet: &str, timeout_ms: u64) -> Self {
        Self {
            subnet: subnet.to_string(),
            timeout_ms,
        }
    }
    
    /// Scan subnet for common services
    pub async fn scan_services(&self) -> Result<Vec<DiscoveredService>> {
        let mut services = Vec::new();
        
        // Scan for Qdrant (port 6333)
        if let Some(addr) = self.scan_port("6333").await {
            services.push(DiscoveredService {
                name: "qdrant".to_string(),
                url: format!("http://{}:6333", addr),
                port: 6333,
            });
        }
        
        // Scan for Ollama (port 11434)
        if let Some(addr) = self.scan_port("11434").await {
            services.push(DiscoveredService {
                name: "ollama".to_string(),
                url: format!("http://{}:11434", addr),
                port: 11434,
            });
        }
        
        // Scan for embedding VM (port 8081)
        if let Some(addr) = self.scan_port("8081").await {
            services.push(DiscoveredService {
                name: "embed_vm".to_string(),
                url: format!("http://{}:8081", addr),
                port: 8081,
            });
        }
        
        Ok(services)
    }
    
    /// Scan a specific port on the subnet
    async fn scan_port(&self, port: &str) -> Option<String> {
        let base_ip = self.subnet.trim_end_matches(".0");
        
        for i in 1..=254 {
            let ip = format!("{}.{}", base_ip, i);
            let addr = format!("{}:{}", ip, port);
            
            if TcpStream::connect_timeout(
                &addr.parse().ok()?,
                Duration::from_millis(self.timeout_ms)
            ).is_ok() {
                return Some(ip);
            }
        }
        
        None
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub name: String,
    pub url: String,
    pub port: u16,
}
```

#### 2.2 Update Configuration with Service Discovery

**File:** `gzmo-core/src/config.rs`

```rust
impl GzmoConfig {
    /// Auto-discover services if not configured
    pub async fn auto_discover_services(&mut self) -> Result<()> {
        let discovery = ServiceDiscovery::new("192.168.31", 500);
        let services = discovery.scan_services().await?;
        
        for service in services {
            match service.name.as_str() {
                "qdrant" => self.services.qdrant = service.url,
                "ollama" => self.services.ollama = service.url,
                "embed_vm" => self.services.embed_vm = service.url,
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

#### 2.3 Update All Network References

**Files to update:**
- `gzmo-core/src/config.rs` (lines 1006, 1175)
- `gzmo-core/src/gateway.rs` (line 1535)
- `gzmo-cli/src/init_cmd.rs` (line 98)
- `gzmo-core/src/scanner.rs` (line 6)

**Pattern:**
```rust
// BEFORE (bad)
let qdrant_url = "http://192.168.31.202:6333";
let ollama_url = "http://192.168.31.110:11434";

// AFTER (good)
let qdrant_url = &config.services.qdrant;
let ollama_url = &config.services.ollama;
```

---

### Phase 3: Hardware Detection (2-3 hours)

**Goal:** Make hardware configuration dynamic

#### 3.1 Create Hardware Detector

**File:** `gzmo-core/src/hardware_detector.rs`

```rust
use std::process::Command;

/// Detect hardware capabilities
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detect GPU information
    pub fn detect_gpu() -> Result<GpuInfo> {
        // Try nvidia-smi first
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&["--query-gpu=name,memory.total", "--format=csv,noheader"])
            .output()
        {
            let lines: Vec<&str> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .collect();
            
            if !lines.is_empty() {
                let parts: Vec<&str> = lines[0].split(',').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    let mem_mb: u32 = parts[1].trim().replace(" MiB", "").parse().unwrap_or(0);
                    return Ok(GpuInfo {
                        name,
                        memory_mb: mem_mb,
                        vendor: "nvidia".to_string(),
                    });
                }
            }
        }
        
        // Try AMD ROCm
        if let Ok(output) = Command::new("rocm-smi")
            .args(&["--showmeminfo", "vram"])
            .output()
        {
            // Parse ROCm output...
            // (implementation depends on ROCm output format)
        }
        
        // No GPU detected
        Ok(GpuInfo {
            name: "None".to_string(),
            memory_mb: 0,
            vendor: "none".to_string(),
        })
    }
    
    /// Detect CPU information
    pub fn detect_cpu() -> CpuInfo {
        let cores = std::thread::available_parallelism()
            .map(|p| p.get() as u32)
            .unwrap_or(4);
        
        CpuInfo {
            cores,
            model: Self::read_cpu_model(),
        }
    }
    
    /// Detect RAM information
    pub fn detect_ram() -> u32 {
        // Read from /proc/meminfo on Linux
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let kb: u64 = line.split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return (kb / 1024 / 1024) as u32; // Convert to GB
                }
            }
        }
        
        8 // Default fallback
    }
    
    /// Read CPU model name
    fn read_cpu_model() -> String {
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("model name") {
                    if let Some(model) = line.split(':').nth(1) {
                        return model.trim().to_string();
                    }
                }
            }
        }
        
        "Unknown".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub memory_mb: u32,
    pub vendor: String,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub cores: u32,
    pub model: String,
}
```

#### 3.2 Update Hardware Configuration

**File:** `gzmo-core/src/config.rs`

```rust
impl GzmoConfig {
    /// Auto-detect hardware and update configuration
    pub fn auto_detect_hardware(&mut self) -> Result<()> {
        let gpu = HardwareDetector::detect_gpu()?;
        let cpu = HardwareDetector::detect_cpu();
        let ram = HardwareDetector::detect_ram();
        
        self.hardware.gpu_memory_gb = gpu.memory_mb / 1024;
        self.hardware.cpu_cores = cpu.cores;
        self.hardware.ram_gb = ram;
        
        Ok(())
    }
}
```

#### 3.3 Update All Hardware References

**Files to update:**
- `gzmo-core/src/stealth.rs` (lines 7, 24)
- `gzmo.toml` (line 165)

**Pattern:**
```rust
// BEFORE (bad)
if cfg!(target_os = "linux") && has_nvidia_gpu() {
    // NVIDIA-specific code
}

// AFTER (good)
if config.hardware.gpu_memory_gb > 0 {
    // GPU-specific code
}
```

---

### Phase 4: Service Name Abstraction (1-2 hours)

**Goal:** Make service names configurable

#### 4.1 Create Service Registry

**File:** `gzmo-core/src/service_registry.rs`

```rust
use std::collections::HashMap;

/// Registry of service names and their endpoints
#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEndpoint>,
}

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub url: String,
    pub description: String,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    /// Register a service
    pub fn register(&mut self, endpoint: ServiceEndpoint) {
        self.services.insert(endpoint.name.clone(), endpoint);
    }
    
    /// Get service URL by name
    pub fn get_url(&self, name: &str) -> Option<String> {
        self.services.get(name).map(|s| s.url.clone())
    }
    
    /// List all registered services
    pub fn list(&self) -> Vec<&ServiceEndpoint> {
        self.services.values().collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

#### 4.2 Update Configuration

**File:** `gzmo-core/src/config.rs`

```rust
impl GzmoConfig {
    /// Load service registry from configuration
    pub fn load_service_registry(&self) -> ServiceRegistry {
        let mut registry = ServiceRegistry::new();
        
        registry.register(ServiceEndpoint {
            name: "qdrant".to_string(),
            url: self.services.qdrant.clone(),
            description: "Vector database for semantic search".to_string(),
        });
        
        registry.register(ServiceEndpoint {
            name: "ollama".to_string(),
            url: self.services.ollama.clone(),
            description: "Local LLM inference".to_string(),
        });
        
        registry.register(ServiceEndpoint {
            name: "embed_vm".to_string(),
            url: self.services.embed_vm.clone(),
            description: "Embedding generation VM".to_string(),
        });
        
        registry
    }
}
```

#### 4.3 Update All Service Name References

**Files to update:**
- `gzmo-cli/src/ingest_eval_cmd.rs` (line 160)
- `gzmo-core/src/memory/ripen.rs` (line 340)

**Pattern:**
```rust
// BEFORE (bad)
let ct101_url = "http://192.168.31.202:6333";
let pve_url = "http://192.168.31.200";

// AFTER (good)
let ct101_url = service_registry.get_url("qdrant").unwrap();
let pve_url = service_registry.get_url("proxmox").unwrap();
```

---

## Migration Script

**File:** `scripts/migrate_to_portable.sh`

```bash
#!/bin/bash
set -e

echo "GZMO Portability Migration"
echo "=========================="
echo ""

# Step 1: Backup current configuration
echo "1. Backing up current configuration..."
cp gzmo.toml gzmo.toml.backup
cp .env .env.backup

# Step 2: Generate new configuration template
echo "2. Generating new configuration template..."
cat > gzmo.toml.new << 'EOF'
# GZMO Configuration
# Copy this to gzmo.toml and customize for your environment

[base]
# Base directory for all GZMO data
base_dir = "."

[data]
vault = "data/vault.db"
lore = "data/lore.toml"

[memory]
episodic = "memory"
dreams = "DREAMS.md"

[wiki]
directory = "wiki"

[skills]
directory = "skills"

[docs]
directory = "docs"

[services]
# Service endpoints (auto-discovered if empty)
qdrant = ""
ollama = ""
embed_vm = ""

[hardware]
# Hardware configuration (auto-detected if empty)
gpu_memory_gb = 0
cpu_cores = 0
ram_gb = 0
EOF

# Step 3: Run auto-discovery
echo "3. Running auto-discovery..."
cargo run -- init --discover

# Step 4: Show migration summary
echo ""
echo "Migration complete!"
echo ""
echo "Next steps:"
echo "1. Review gzmo.toml and adjust as needed"
echo "2. Run 'cargo run -- init' to initialize"
echo "3. Test with 'cargo run -- health'"
echo ""
echo "For more information, see docs/PORTABILITY_REFACTORING.md"
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = GzmoConfig::default();
        assert_eq!(config.services.qdrant, "http://localhost:6333");
        assert_eq!(config.services.ollama, "http://localhost:11434");
    }
    
    #[test]
    fn test_path_resolution() {
        let config = GzmoConfig {
            base_dir: PathBuf::from("/tmp/gzmo"),
            ..GzmoConfig::default()
        };
        
        assert_eq!(config.vault_path(), PathBuf::from("/tmp/gzmo/data/vault.db"));
        assert_eq!(config.memory_dir(), PathBuf::from("/tmp/gzmo/memory"));
    }
}

#[cfg(test)]
mod hardware_tests {
    use super::*;
    
    #[test]
    fn test_gpu_detection() {
        let gpu = HardwareDetector::detect_gpu().unwrap();
        // Should not panic, may return "None" on systems without GPU
        assert!(!gpu.name.is_empty());
    }
    
    #[test]
    fn test_cpu_detection() {
        let cpu = HardwareDetector::detect_cpu();
        assert!(cpu.cores > 0);
        assert!(!cpu.model.is_empty());
    }
    
    #[test]
    fn test_ram_detection() {
        let ram = HardwareDetector::detect_ram();
        assert!(ram > 0);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_service_discovery() {
    let discovery = ServiceDiscovery::new("127.0.0.1", 100);
    let services = discovery.scan_services().await.unwrap();
    
    // Should find at least localhost services
    assert!(!services.is_empty());
}

#[test]
fn test_config_load_save() {
    let config = GzmoConfig::default();
    let path = PathBuf::from("/tmp/test_gzmo.toml");
    
    config.save(&path).unwrap();
    let loaded = GzmoConfig::load_or_create(&path).unwrap();
    
    assert_eq!(loaded.services.qdrant, config.services.qdrant);
    
    // Cleanup
    std::fs::remove_file(&path).ok();
}
```

### Manual Testing

1. **Test on different machine**
   ```bash
   git clone <repo>
   cd GZMO
   cargo run -- init --discover
   cargo run -- health
   ```

2. **Test with custom configuration**
   ```bash
   cargo run -- init --config /path/to/custom.toml
   cargo run -- health
   ```

3. **Test with different hardware**
   - Run on machine with GPU
   - Run on machine without GPU
   - Verify auto-detection works

---

## Success Criteria

### Phase 1 (Paths)

- [ ] All absolute paths removed from source code
- [ ] Configuration system loads from file
- [ ] Paths are relative to base_dir
- [ ] Default paths work on any machine

### Phase 2 (Network)

- [ ] All hardcoded IPs removed
- [ ] Service discovery works
- [ ] Configuration supports custom endpoints
- [ ] Auto-discovery finds services on local network

### Phase 3 (Hardware)

- [ ] Hardware detection works on Linux
- [ ] Configuration auto-detects GPU/CPU/RAM
- [ ] Code works with any GPU vendor
- [ ] Graceful fallback when hardware not detected

### Phase 4 (Services)

- [ ] Service names are configurable
- [ ] Service registry abstracts endpoints
- [ ] No hardcoded service names in code
- [ ] Easy to add new services

### Overall

- [ ] Can run on any Linux machine without code changes
- [ ] Configuration file documents all options
- [ ] Migration script works
- [ ] Tests pass on CI
- [ ] Documentation updated

---

## Migration Checklist

### Before Refactoring

- [ ] Backup current gzmo.toml
- [ ] Backup current .env
- [ ] Document current service endpoints
- [ ] Document current directory structure

### During Refactoring

- [ ] Implement configuration system
- [ ] Implement service discovery
- [ ] Implement hardware detection
- [ ] Update all path references
- [ ] Update all network references
- [ ] Update all hardware references
- [ ] Update all service name references

### After Refactoring

- [ ] Run all tests
- [ ] Test on current machine
- [ ] Test on different machine
- [ ] Test with custom configuration
- [ ] Update documentation
- [ ] Create migration script
- [ ] Update README

---

## References

- [Configuration System Design](docs/CONFIGURATION_DESIGN.md)
- [Service Discovery Protocol](docs/SERVICE_DISCOVERY.md)
- [Hardware Detection API](docs/HARDWARE_DETECTION.md)
- [Portability Best Practices](docs/PORTABILITY_BEST_PRACTICES.md)

---

**Last Updated:** 2026-07-09  
**Author:** GZMO Development Team  
**Status:** Ready for Implementation
