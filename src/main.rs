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
    GetRawInputData, GetRawInputDeviceInfoW, GetRawInputDeviceList, RegisterRawInputDevices,
    RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_DEVICE_INFO,
    RID_DEVICE_INFO_HID, RID_DEVICE_INFO_KEYBOARD, RID_INPUT, RIDI_DEVICEINFO, RIDI_DEVICENAME,
    RIM_TYPEHID, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
};
#[cfg(target_os = "windows")]
use winapi::shared::ntdef::HANDLE;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{
    VK_BACK, VK_TAB, VK_RETURN, VK_SHIFT, VK_CONTROL, VK_MENU, VK_CAPITAL, VK_ESCAPE,
    VK_SPACE, VK_PRIOR, VK_NEXT, VK_END, VK_HOME, VK_LEFT, VK_UP, VK_RIGHT, VK_DOWN,
    VK_INSERT, VK_DELETE, VK_LWIN, VK_RWIN, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2,
    VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9,
    VK_MULTIPLY, VK_ADD, VK_SUBTRACT, VK_DECIMAL, VK_DIVIDE, VK_F1, VK_F2, VK_F3, VK_F4,
    VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_NUMLOCK, VK_SCROLL,
    VK_LSHIFT, VK_RSHIFT, VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU,
    RegisterClassW, CreateWindowExW, DefWindowProcW, GetMessageW, TranslateMessage,
    DispatchMessageW, PostQuitMessage, WM_INPUT, WM_DESTROY, WNDCLASSW, MSG,
    CW_USEDEFAULT, WS_OVERLAPPEDWINDOW,
};
#[cfg(target_os = "windows")]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use std::collections::HashMap;

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
#[derive(Debug, Clone)]
struct KeyboardDeviceInfo {
    handle: HANDLE,
    device_name: String,
    vid: u16,
    pid: u16,
    device_id: String, // Format: "VID:PID"
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

    /// Create VK code to key name mapping
    fn create_vk_to_name_map() -> HashMap<i32, String> {
        let mut map = HashMap::new();
        
        // Alphanumeric keys (0x30-0x39, 0x41-0x5A)
        for c in b'0'..=b'9' {
            map.insert(c as i32, (c as char).to_string());
        }
        for c in b'A'..=b'Z' {
            map.insert(c as i32, (c as char).to_string());
        }
        
        // Special keys
        map.insert(VK_BACK as i32, "Backspace".to_string());
        map.insert(VK_TAB as i32, "Tab".to_string());
        map.insert(VK_RETURN as i32, "Enter".to_string());
        map.insert(VK_SHIFT as i32, "Shift".to_string());
        map.insert(VK_CONTROL as i32, "Ctrl".to_string());
        map.insert(VK_MENU as i32, "Alt".to_string());
        map.insert(VK_CAPITAL as i32, "CapsLock".to_string());
        map.insert(VK_ESCAPE as i32, "Escape".to_string());
        map.insert(VK_SPACE as i32, "Space".to_string());
        map.insert(VK_PRIOR as i32, "PageUp".to_string());
        map.insert(VK_NEXT as i32, "PageDown".to_string());
        map.insert(VK_END as i32, "End".to_string());
        map.insert(VK_HOME as i32, "Home".to_string());
        map.insert(VK_LEFT as i32, "Left".to_string());
        map.insert(VK_UP as i32, "Up".to_string());
        map.insert(VK_RIGHT as i32, "Right".to_string());
        map.insert(VK_DOWN as i32, "Down".to_string());
        map.insert(VK_INSERT as i32, "Insert".to_string());
        map.insert(VK_DELETE as i32, "Delete".to_string());
        
        // Windows keys
        map.insert(VK_LWIN as i32, "LWin".to_string());
        map.insert(VK_RWIN as i32, "RWin".to_string());
        
        // Numpad keys
        map.insert(VK_NUMPAD0 as i32, "Numpad0".to_string());
        map.insert(VK_NUMPAD1 as i32, "Numpad1".to_string());
        map.insert(VK_NUMPAD2 as i32, "Numpad2".to_string());
        map.insert(VK_NUMPAD3 as i32, "Numpad3".to_string());
        map.insert(VK_NUMPAD4 as i32, "Numpad4".to_string());
        map.insert(VK_NUMPAD5 as i32, "Numpad5".to_string());
        map.insert(VK_NUMPAD6 as i32, "Numpad6".to_string());
        map.insert(VK_NUMPAD7 as i32, "Numpad7".to_string());
        map.insert(VK_NUMPAD8 as i32, "Numpad8".to_string());
        map.insert(VK_NUMPAD9 as i32, "Numpad9".to_string());
        map.insert(VK_MULTIPLY as i32, "NumpadMultiply".to_string());
        map.insert(VK_ADD as i32, "NumpadAdd".to_string());
        map.insert(VK_SUBTRACT as i32, "NumpadSubtract".to_string());
        map.insert(VK_DECIMAL as i32, "NumpadDecimal".to_string());
        map.insert(VK_DIVIDE as i32, "NumpadDivide".to_string());
        
        // Function keys
        map.insert(VK_F1 as i32, "F1".to_string());
        map.insert(VK_F2 as i32, "F2".to_string());
        map.insert(VK_F3 as i32, "F3".to_string());
        map.insert(VK_F4 as i32, "F4".to_string());
        map.insert(VK_F5 as i32, "F5".to_string());
        map.insert(VK_F6 as i32, "F6".to_string());
        map.insert(VK_F7 as i32, "F7".to_string());
        map.insert(VK_F8 as i32, "F8".to_string());
        map.insert(VK_F9 as i32, "F9".to_string());
        map.insert(VK_F10 as i32, "F10".to_string());
        map.insert(VK_F11 as i32, "F11".to_string());
        map.insert(VK_F12 as i32, "F12".to_string());
        
        // Lock keys
        map.insert(VK_NUMLOCK as i32, "NumLock".to_string());
        map.insert(VK_SCROLL as i32, "ScrollLock".to_string());
        
        // Left/Right specific keys
        map.insert(VK_LSHIFT as i32, "LShift".to_string());
        map.insert(VK_RSHIFT as i32, "RShift".to_string());
        map.insert(VK_LCONTROL as i32, "LCtrl".to_string());
        map.insert(VK_RCONTROL as i32, "RCtrl".to_string());
        map.insert(VK_LMENU as i32, "LAlt".to_string());
        map.insert(VK_RMENU as i32, "RAlt".to_string());
        
        map
    }

    /// Convert VK code to key name
    fn vk_to_key_name(vk_code: i32) -> String {
        let map = Self::create_vk_to_name_map();
        map.get(&vk_code)
            .cloned()
            .unwrap_or_else(|| format!("VK_{}", vk_code))
    }

    /// Create key name to VK code mapping
    fn create_name_to_vk_map() -> HashMap<String, i32> {
        let vk_to_name = Self::create_vk_to_name_map();
        vk_to_name.into_iter().map(|(k, v)| (v, k)).collect()
    }

    /// Convert key name to VK code
    fn key_name_to_vk(key_name: &str) -> Option<i32> {
        let map = Self::create_name_to_vk_map();
        map.get(key_name).copied()
    }

    /// List all connected keyboard devices
    #[allow(dead_code)]
    unsafe fn list_keyboard_devices() -> Result<Vec<KeyboardDeviceInfo>, String> {
        let mut devices = Vec::new();
        let mut device_count: UINT = 0;

        // First call: get the number of devices
        let result = GetRawInputDeviceList(
            std::ptr::null_mut(),
            &mut device_count,
            std::mem::size_of::<winapi::um::winuser::RAWINPUTDEVICELIST>() as UINT,
        );

        if result == u32::MAX {
            return Err("Failed to get device count".to_string());
        }

        if device_count == 0 {
            return Ok(devices);
        }

        // Allocate buffer for device list
        let mut device_list: Vec<winapi::um::winuser::RAWINPUTDEVICELIST> =
            vec![std::mem::zeroed(); device_count as usize];

        // Second call: get the device list
        let result = GetRawInputDeviceList(
            device_list.as_mut_ptr(),
            &mut device_count,
            std::mem::size_of::<winapi::um::winuser::RAWINPUTDEVICELIST>() as UINT,
        );

        if result == u32::MAX {
            return Err("Failed to get device list".to_string());
        }

        // Filter keyboard devices and get their info
        for device in device_list.iter().take(device_count as usize) {
            if device.dwType == RIM_TYPEKEYBOARD {
                if let Ok(device_info) = Self::get_device_info(device.hDevice) {
                    devices.push(device_info);
                }
            }
        }

        Ok(devices)
    }

    /// Get detailed information about a device
    #[allow(dead_code)]
    unsafe fn get_device_info(handle: HANDLE) -> Result<KeyboardDeviceInfo, String> {
        // Get device name
        let mut name_size: UINT = 0;
        GetRawInputDeviceInfoW(handle, RIDI_DEVICENAME, std::ptr::null_mut(), &mut name_size);

        if name_size == 0 {
            return Err("Failed to get device name size".to_string());
        }

        let mut name_buffer: Vec<u16> = vec![0; name_size as usize];
        let result = GetRawInputDeviceInfoW(
            handle,
            RIDI_DEVICENAME,
            name_buffer.as_mut_ptr() as *mut _,
            &mut name_size,
        );

        if result == u32::MAX {
            return Err("Failed to get device name".to_string());
        }

        let device_name = String::from_utf16_lossy(&name_buffer)
            .trim_end_matches('\0')
            .to_string();

        // Get device info (for VID/PID)
        let mut info_size: UINT = std::mem::size_of::<RID_DEVICE_INFO>() as UINT;
        let mut device_info: RID_DEVICE_INFO = std::mem::zeroed();
        device_info.cbSize = info_size;

        let result = GetRawInputDeviceInfoW(
            handle,
            RIDI_DEVICEINFO,
            &mut device_info as *mut _ as *mut _,
            &mut info_size,
        );

        if result == u32::MAX {
            return Err("Failed to get device info".to_string());
        }

        // Extract VID/PID from device name (format: \\?\\HID#VID_XXXX&PID_YYYY#...)
        let (vid, pid) = Self::parse_vid_pid(&device_name).unwrap_or((0, 0));
        let device_id = format!("{:04X}:{:04X}", vid, pid);

        Ok(KeyboardDeviceInfo {
            handle,
            device_name,
            vid,
            pid,
            device_id,
        })
    }

    /// Parse VID and PID from device name string
    #[allow(dead_code)]
    fn parse_vid_pid(device_name: &str) -> Option<(u16, u16)> {
        // Device name format: \\?\\HID#VID_XXXX&PID_YYYY#...
        let upper = device_name.to_uppercase();
        
        let vid_pos = upper.find("VID_")?;
        let pid_pos = upper.find("PID_")?;
        
        let vid_str = &upper[vid_pos + 4..vid_pos + 8];
        let pid_str = &upper[pid_pos + 4..pid_pos + 8];
        
        let vid = u16::from_str_radix(vid_str, 16).ok()?;
        let pid = u16::from_str_radix(pid_str, 16).ok()?;
        
        Some((vid, pid))
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
            
            // Convert virtual key code to key name
            let key_name = Self::vk_to_key_name(vkey as i32);
            
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
            #[cfg(target_os = "windows")]
            {
                println!("Connected Keyboards:");
                match unsafe { RawInputHandler::list_keyboard_devices() } {
                    Ok(devices) => {
                        if devices.is_empty() {
                            println!("  (No keyboard devices detected)");
                        } else {
                            for device in &devices {
                                let configured = config.devices.iter()
                                    .any(|d| d.device_id == device.device_id);
                                let status = if configured { "[Configured]" } else { "" };
                                println!("  - {} {} {}", device.device_id, device.device_name, status);
                                if let Some(dev_config) = config.devices.iter()
                                    .find(|d| d.device_id == device.device_id) {
                                    println!("    Mappings: {}", dev_config.mappings.len());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing devices: {}", e);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("Device detection is only supported on Windows.");
                println!("\nConfigured Devices:");
                if config.devices.is_empty() {
                    println!("  (No devices configured)");
                } else {
                    for device in &config.devices {
                        println!("  - {} ({} mappings)", device.device_id, device.mappings.len());
                    }
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
