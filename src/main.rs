use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "KeyboardRemapperR")]
#[command(about = "Lightweight device-specific keyboard remapper for Windows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List connected keyboards
    List,
    /// Set key mapping for a device
    Set {
        /// Device ID (VID:PID)
        device_id: String,
        /// Source key
        from_key: String,
        /// Target key
        to_key: String,
        /// Mapping type: remap, swap, or disable
        #[arg(short, long, default_value = "remap")]
        mode: String,
    },
    /// Remove key mapping
    Remove {
        /// Device ID (VID:PID)
        device_id: String,
        /// Source key
        from_key: String,
    },
    /// List mappings for a device
    Show {
        /// Device ID (VID:PID)
        device_id: String,
    },
    /// Save configuration to file
    Save {
        /// Output file path
        #[arg(short, long, default_value = "config.json")]
        output: PathBuf,
    },
    /// Load configuration from file
    Load {
        /// Input file path
        #[arg(short, long, default_value = "config.json")]
        input: PathBuf,
    },
    /// Start keyboard remapping service
    Start,
    /// Stop keyboard remapping service
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyMapping {
    from_key: String,
    to_key: String,
    mode: String, // "remap", "swap", "disable"
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceConfig {
    device_id: String,
    device_name: String,
    mappings: Vec<KeyMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    devices: HashMap<String, DeviceConfig>,
}

impl Config {
    fn new() -> Self {
        Config {
            devices: HashMap::new(),
        }
    }

    fn add_device(&mut self, device_id: String, device_name: String) {
        self.devices.insert(
            device_id.clone(),
            DeviceConfig {
                device_id,
                device_name,
                mappings: Vec::new(),
            },
        );
    }

    fn add_mapping(&mut self, device_id: &str, mapping: KeyMapping) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.mappings.push(mapping);
        }
    }

    fn remove_mapping(&mut self, device_id: &str, from_key: &str) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.mappings.retain(|m| m.from_key != from_key);
        }
    }

    fn save(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(path, json)?;
        println!("Configuration saved to: {}", path.display());
        Ok(())
    }

    fn load(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let config = serde_json::from_str(&json)?;
        println!("Configuration loaded from: {}", path.display());
        Ok(config)
    }
}

struct KeyboardManager {
    config: Config,
}

impl KeyboardManager {
    fn new() -> Self {
        KeyboardManager {
            config: Config::new(),
        }
    }

    fn list_devices(&self) {
        println!("Devices:");
        
        // Simulated device detection
        let devices = vec![
            ("04FE:0021", "HHKB Professional"),
            ("0000:0000", "Built-in Keyboard"),
            ("1234:5678", "External USB Keyboard"),
        ];

        for (device_id, device_name) in devices {
            println!("Device ID: {}", device_id);
            println!("  Name: {}", device_name);
            
            if let Some(config) = self.config.devices.get(device_id) {
                println!("  Mappings: {}", config.mappings.len());
                for mapping in &config.mappings {
                    println!("    {} -> {} ({})", mapping.from_key, mapping.to_key, mapping.mode);
                }
            } else {
                println!("  Mappings: 0");
            }
            println!();
        }
    }

    fn show_device(&self, device_id: &str) {
        if let Some(device) = self.config.devices.get(device_id) {
            println!("Device: {}", device.device_id);
            println!("Mappings:");
            
            if device.mappings.is_empty() {
                println!("  No mappings configured");
            } else {
                for (i, mapping) in device.mappings.iter().enumerate() {
                    println!("  {}. {} -> {} ({})", 
                        i + 1, 
                        mapping.from_key, 
                        mapping.to_key, 
                        mapping.mode
                    );
                }
            }
            println!();
        } else {
            println!("Device {} not found", device_id);
        }
    }

    fn process_key_event(&self, device_id: &str, key: &str, _is_pressed: bool) -> Option<String> {
        if let Some(device) = self.config.devices.get(device_id) {
            for mapping in &device.mappings {
                if mapping.from_key == key {
                    match mapping.mode.as_str() {
                        "remap" => return Some(mapping.to_key.clone()),
                        "swap" => return Some(mapping.to_key.clone()),
                        "disable" => return Some("NONE".to_string()),
                        _ => {}
                    }
                }
            }
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut manager = KeyboardManager::new();

    match cli.command {
        Commands::List => {
            manager.list_devices();
        }
        Commands::Set {
            device_id,
            from_key,
            to_key,
            mode,
        } => {
            manager.config.add_device(device_id.clone(), format!("Device {}", device_id));
            let mapping = KeyMapping {
                from_key: from_key.clone(),
                to_key: to_key.clone(),
                mode: mode.clone(),
            };
            manager.config.add_mapping(&device_id, mapping);
            println!(
                "Mapping set successfully: {} -> {} ({}) for device {}",
                from_key, to_key, mode, device_id
            );
        }
        Commands::Remove { device_id, from_key } => {
            manager.config.remove_mapping(&device_id, &from_key);
            println!("Mapping removed successfully: {} for device {}", from_key, device_id);
        }
        Commands::Show { device_id } => {
            manager.show_device(&device_id);
        }
        Commands::Save { output } => {
            manager.config.save(&output)?;
            println!("Configuration saved successfully to {}", output.display());
        }
        Commands::Load { input } => {
            manager.config = Config::load(&input)?;
            println!("Configuration loaded successfully from {}", input.display());
        }
        Commands::Start => {
            println!("Starting keyboard remapping service...");
            println!("Note: This is a CLI tool. For actual remapping, use on Windows with admin rights.");
            println!("Service started. Press Ctrl+C to stop.");
            std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
        }
        Commands::Stop => {
            println!("Stopping keyboard remapping service...");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_device() {
        let mut config = Config::new();
        config.add_device("04FE:0021".to_string(), "HHKB".to_string());
        assert_eq!(config.devices.len(), 1);
    }

    #[test]
    fn test_add_mapping() {
        let mut config = Config::new();
        config.add_device("04FE:0021".to_string(), "HHKB".to_string());
        let mapping = KeyMapping {
            from_key: "CapsLock".to_string(),
            to_key: "LCtrl".to_string(),
            mode: "swap".to_string(),
        };
        config.add_mapping("04FE:0021", mapping);
        assert_eq!(config.devices["04FE:0021"].mappings.len(), 1);
    }

    #[test]
    fn test_remove_mapping() {
        let mut config = Config::new();
        config.add_device("04FE:0021".to_string(), "HHKB".to_string());
        let mapping = KeyMapping {
            from_key: "CapsLock".to_string(),
            to_key: "LCtrl".to_string(),
            mode: "swap".to_string(),
        };
        config.add_mapping("04FE:0021", mapping);
        config.remove_mapping("04FE:0021", "CapsLock");
        assert_eq!(config.devices["04FE:0021"].mappings.len(), 0);
    }
}
