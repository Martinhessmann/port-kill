use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub discovery: DiscoveryConfig,
    pub ports: PortsConfig,
    pub ignore: IgnoreConfig,
    pub app: AppConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    /// Discovery mode: "range", "specific", or "all"
    pub mode: DiscoveryMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoveryMode {
    Range,
    Specific,
    All,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortsConfig {
    /// Port ranges to monitor (only used when mode = "range")
    pub ranges: Vec<PortRange>,
    /// Specific ports to monitor (only used when mode = "specific")
    pub specific: Vec<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IgnoreConfig {
    /// Ports to ignore (applies to all discovery modes)
    pub ports: Vec<u16>,
    /// Process names to ignore (applies to all discovery modes)
    pub processes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Monitoring interval in seconds
    pub monitoring_interval_seconds: u64,
    /// Enable verbose logging
    pub verbose_logging: bool,
    /// Show process IDs in output
    pub show_process_ids: bool,
    /// Menu update cooldown in seconds
    pub menu_update_cooldown_seconds: u64,
    /// Maximum number of processes to show in menu (for stability)
    pub max_processes_in_menu: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discovery: DiscoveryConfig {
                mode: DiscoveryMode::Range,
            },
            ports: PortsConfig {
                ranges: vec![
                    PortRange {
                        start: 3000,
                        end: 3010,
                        description: "React, Next.js, development servers".to_string(),
                    },
                    PortRange {
                        start: 5000,
                        end: 5010,
                        description: "Flask, Vite, PostgreSQL, development".to_string(),
                    },
                    PortRange {
                        start: 8000,
                        end: 8010,
                        description: "Django, FastAPI, general HTTP servers".to_string(),
                    },
                ],
                specific: vec![3000, 3001, 5000, 5173, 8000, 8080],
            },
            ignore: IgnoreConfig {
                ports: vec![5353, 7000],
                processes: vec![
                    "Google".to_string(),
                    "Adobe".to_string(),
                    "Dropbox".to_string(),
                    "Cursor".to_string(),
                    "Figma".to_string(),
                    "Raycast".to_string(),
                    "ControlCe".to_string(),
                    "sharingd".to_string(),
                    "rapportd".to_string(),
                ],
            },
            app: AppConfig {
                monitoring_interval_seconds: 3,
                verbose_logging: false,
                show_process_ids: false,
                menu_update_cooldown_seconds: 2,
                max_processes_in_menu: 20,
            },
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if it doesn't exist
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            log::info!("Config file not found at {:?}, creating default configuration", path);
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        log::info!("Loaded configuration from {:?}", path);
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        log::info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get all ports to monitor based on configuration
    pub fn get_ports_to_monitor(&self) -> Vec<u16> {
        match self.discovery.mode {
            DiscoveryMode::Range => {
                let mut ports = Vec::new();
                for range in &self.ports.ranges {
                    ports.extend(range.start..=range.end);
                }
                ports
            }
            DiscoveryMode::Specific => self.ports.specific.clone(),
            DiscoveryMode::All => {
                // For "all" mode, return empty vec to indicate full discovery
                Vec::new()
            }
        }
    }

    /// Get ports to ignore as a HashSet for efficient lookup
    pub fn get_ignore_ports_set(&self) -> HashSet<u16> {
        self.ignore.ports.iter().cloned().collect()
    }

    /// Get process names to ignore as a HashSet for efficient lookup
    pub fn get_ignore_processes_set(&self) -> HashSet<String> {
        self.ignore.processes.iter().cloned().collect()
    }

    /// Check if discovery mode is "all"
    pub fn is_discover_all(&self) -> bool {
        matches!(self.discovery.mode, DiscoveryMode::All)
    }

    /// Get description of current monitoring configuration
    pub fn get_monitoring_description(&self) -> String {
        match self.discovery.mode {
            DiscoveryMode::All => "auto-discovering ALL listening processes on ANY port".to_string(),
            DiscoveryMode::Specific => {
                format!("specific ports: {}",
                    self.ports.specific.iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<_>>()
                        .join(", "))
            }
            DiscoveryMode::Range => {
                let ranges_desc = self.ports.ranges.iter()
                    .map(|r| format!("{}-{} ({})", r.start, r.end, r.description))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("port ranges: {}", ranges_desc)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.discovery.mode, DiscoveryMode::Range);
        assert!(!config.get_ports_to_monitor().is_empty());
        assert!(!config.is_discover_all());
    }

    #[test]
    fn test_get_ports_to_monitor_range() {
        let config = Config {
            discovery: DiscoveryConfig { mode: DiscoveryMode::Range },
            ports: PortsConfig {
                ranges: vec![
                    PortRange { start: 3000, end: 3002, description: "Test".to_string() },
                    PortRange { start: 8000, end: 8001, description: "Test".to_string() },
                ],
                specific: vec![],
            },
            ignore: IgnoreConfig { ports: vec![], processes: vec![] },
            app: AppConfig::default(),
        };

        let ports = config.get_ports_to_monitor();
        assert_eq!(ports, vec![3000, 3001, 3002, 8000, 8001]);
    }

    #[test]
    fn test_get_ports_to_monitor_specific() {
        let config = Config {
            discovery: DiscoveryConfig { mode: DiscoveryMode::Specific },
            ports: PortsConfig {
                ranges: vec![],
                specific: vec![3000, 8080],
            },
            ignore: IgnoreConfig { ports: vec![], processes: vec![] },
            app: AppConfig::default(),
        };

        let ports = config.get_ports_to_monitor();
        assert_eq!(ports, vec![3000, 8080]);
    }

    #[test]
    fn test_is_discover_all() {
        let config = Config {
            discovery: DiscoveryConfig { mode: DiscoveryMode::All },
            ports: PortsConfig { ranges: vec![], specific: vec![] },
            ignore: IgnoreConfig { ports: vec![], processes: vec![] },
            app: AppConfig::default(),
        };

        assert!(config.is_discover_all());
        assert!(config.get_ports_to_monitor().is_empty());
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_seconds: 3,
            verbose_logging: false,
            show_process_ids: false,
            menu_update_cooldown_seconds: 2,
            max_processes_in_menu: 20,
        }
    }
}
