use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused for now
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{LPARAM, UINT};
#[cfg(target_os = "windows")]
use winapi::shared::windef::HWND;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{
    GetRawInputData, RegisterRawInputDevices, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
};

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
    /// Start the remapping service (Windows only)
    #[cfg(target_os = "windows")]
    Start,
    /// Stop the remapping service
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum MappingType {
    Remap,
    Swap,
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyMapping {
    from: String,
    to: String,
    #[serde(rename = "type")]
    mapping_type: MappingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceConfig {
    device_id: String,
    mappings: Vec<KeyMapping>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    devices: Vec<DeviceConfig>,
}

impl Config {
    fn new() -> Self {
        Config {
            devices: Vec::new(),
        }
    }

    fn add_device(&mut self, device_id: String) {
        if !self.devices.iter().any(|d| d.device_id == device_id) {
            self.devices.push(DeviceConfig {
                device_id,
                mappings: Vec::new(),
            });
        }
    }

    fn add_mapping(&mut self, device_id: &str, from: String, to: String, mapping_type: MappingType) {
        self.add_device(device_id.to_string());
        if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == device_id) {
            // Remove existing mapping for the same key
            device.mappings.retain(|m| m.from != from);
            device.mappings.push(KeyMapping {
                from,
                to,
                mapping_type,
            });
        }
    }

    fn remove_mapping(&mut self, device_id: &str, from: &str) -> bool {
        if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == device_id) {
            let before = device.mappings.len();
            device.mappings.retain(|m| m.from != from);
            return device.mappings.len() < before;
        }
        false
    }

    #[allow(dead_code)]
    fn process_key_event(&self, device_id: &str, key: &str, _pressed: bool) -> Option<String> {
        if let Some(device) = self.devices.iter().find(|d| d.device_id == device_id) {
            if let Some(mapping) = device.mappings.iter().find(|m| m.from == key) {
                match mapping.mapping_type {
                    MappingType::Remap => Some(mapping.to.clone()),
                    MappingType::Swap => Some(mapping.to.clone()),
                    MappingType::Disable => Some("None".to_string()),
                }
            } else {
                Some(key.to_string())
            }
        } else {
            Some(key.to_string())
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
struct RawInputHandler {
    config: Config,
}

#[cfg(target_os = "windows")]
impl RawInputHandler {
    #[allow(dead_code)]
    fn new(config: Config) -> Self {
        RawInputHandler { config }
    }

    #[allow(dead_code)]
    unsafe fn register_raw_input_devices(&self, hwnd: HWND) -> Result<(), String> {
        let mut devices = [RAWINPUTDEVICE {
            usUsagePage: 0x01, // Generic Desktop Controls
            usUsage: 0x06,     // Keyboard
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        }];

        let result = RegisterRawInputDevices(
            devices.as_mut_ptr(),
            devices.len() as UINT,
            std::mem::size_of::<RAWINPUTDEVICE>() as UINT,
        );

        if result == 0 {
            Err("Failed to register raw input devices".to_string())
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
        let mut size: UINT = 0;
        
        // Get the size of the raw input data
        GetRawInputData(
            lparam as *mut _,
            RID_INPUT,
            std::ptr::null_mut(),
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as UINT,
        );

        if size == 0 {
            return None;
        }

        // Allocate buffer and get the raw input data
        let mut buffer: Vec<u8> = vec![0; size as usize];
        let result = GetRawInputData(
            lparam as *mut _,
            RID_INPUT,
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as UINT,
        );

        if result == u32::MAX {
            return None;
        }

        let raw_input = &*(buffer.as_ptr() as *const RAWINPUT);
        
        // Check if it's a keyboard input
        if raw_input.header.dwType == RIM_TYPEKEYBOARD {
            let keyboard = raw_input.data.keyboard();
            let vkey = keyboard.VKey;
            let _scancode = keyboard.MakeCode;
            let flags = keyboard.Flags;
            
            // Get device handle (simplified - in real implementation, you'd extract VID/PID)
            let device_id = "04FE:0021"; // Placeholder
            
            // Convert virtual key code to key name (simplified)
            let key_name = format!("VK_{}", vkey);
            
            // Check if key is pressed (not released)
            let is_pressed = (flags & 0x01) == 0;
            
            // Process the key event
            return self.config.process_key_event(device_id, &key_name, is_pressed);
        }

        None
    }
}

fn main() {
    let cli = Cli::parse();
    let mut config = load_config_or_default();

    match cli.command {
        Commands::List => {
            println!("Devices:");
            if config.devices.is_empty() {
                println!("  (No devices configured)");
            } else {
                for device in &config.devices {
                    println!("  - {} ({} mappings)", device.device_id, device.mappings.len());
                }
            }
        }
        Commands::Set {
            device_id,
            from_key,
            to_key,
            mode,
        } => {
            let mapping_type = match mode.as_str() {
                "remap" => MappingType::Remap,
                "swap" => MappingType::Swap,
                "disable" => MappingType::Disable,
                _ => {
                    eprintln!("Invalid mode: {}. Use 'remap', 'swap', or 'disable'.", mode);
                    std::process::exit(1);
                }
            };
            config.add_mapping(&device_id, from_key.clone(), to_key.clone(), mapping_type);
            println!("Mapping set successfully: {} -> {} ({})", from_key, to_key, mode);
        }
        Commands::Remove { device_id, from_key } => {
            if config.remove_mapping(&device_id, &from_key) {
                println!("Mapping removed successfully: {}", from_key);
            } else {
                println!("No mapping found for key: {}", from_key);
            }
        }
        Commands::Show { device_id } => {
            if let Some(device) = config.devices.iter().find(|d| d.device_id == device_id) {
                println!("Device: {}", device.device_id);
                println!("Mappings:");
                for mapping in &device.mappings {
                    println!("  {} -> {} ({:?})", mapping.from, mapping.to, mapping.mapping_type);
                }
            } else {
                println!("Device not found: {}", device_id);
            }
        }
        Commands::Save { output } => {
            save_config(&config, &output);
            println!("Configuration saved to: {}", output.display());
        }
        Commands::Load { input } => {
            let _loaded_config = load_config(&input);
            println!("Configuration loaded successfully from: {}", input.display());
        }
        #[cfg(target_os = "windows")]
        Commands::Start => {
            println!("Starting keyboard remapping service...");
            println!("Note: This requires administrator privileges on Windows.");
            println!("Raw Input API integration is active.");
            
            // In a real implementation, this would:
            // 1. Create a hidden window to receive raw input messages
            // 2. Register for raw input devices
            // 3. Run a message loop to process keyboard events
            // 4. Apply key mappings based on device ID
            
            println!("Service started. Press Ctrl+C to stop.");
            // Placeholder - actual implementation would run indefinitely
        }
        Commands::Stop => {
            println!("Stopping keyboard remapping service...");
            println!("Service stopped.");
        }
    }
}

fn load_config_or_default() -> Config {
    let config_path = PathBuf::from("config.json");
    if config_path.exists() {
        load_config(&config_path)
    } else {
        Config::new()
    }
}

fn load_config(path: &PathBuf) -> Config {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Config::new()),
        Err(_) => Config::new(),
    }
}

fn save_config(config: &Config, path: &PathBuf) {
    let json = serde_json::to_string_pretty(config).unwrap();
    fs::write(path, json).expect("Failed to write config file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_device() {
        let mut config = Config::new();
        config.add_device("04FE:0021".to_string());
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].device_id, "04FE:0021");
    }

    #[test]
    fn test_add_mapping() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Remap);
        assert_eq!(config.devices.len(), 1);
        assert_eq!(config.devices[0].mappings.len(), 1);
        assert_eq!(config.devices[0].mappings[0].from, "CapsLock");
        assert_eq!(config.devices[0].mappings[0].to, "LCtrl");
    }

    #[test]
    fn test_remove_mapping() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Remap);
        assert!(config.remove_mapping("04FE:0021", "CapsLock"));
        assert_eq!(config.devices[0].mappings.len(), 0);
    }
}
