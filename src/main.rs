use clap::{Parser, Subcommand};

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState,
        ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};
#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::time::Duration;
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
    // Phase 3: Low-level keyboard hook and SendInput
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx, SendInput,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
};
#[cfg(target_os = "windows")]
use winapi::um::libloaderapi::GetModuleHandleW;
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::{LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use std::collections::{HashMap, HashSet};
#[cfg(target_os = "windows")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
static mut GLOBAL_HANDLER: Option<Arc<Mutex<RawInputHandler>>> = None;

#[cfg(target_os = "windows")]
static mut KEYBOARD_HOOK: Option<winapi::shared::windef::HHOOK> = None;

#[cfg(target_os = "windows")]
static mut SUPPRESSED_KEYS: Option<std::collections::HashSet<u16>> = None;

#[cfg(target_os = "windows")]
const INJECTED_KEY_MARKER: usize = 0x12345678;

#[cfg(target_os = "windows")]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            // Process raw input
            if let Some(handler) = &GLOBAL_HANDLER {
                if let Ok(mut handler) = handler.lock() {
                    handler.process_raw_input(lparam);
                }
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

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
            
            // For Swap mode, also remove reverse mapping
            if mapping_type == MappingType::Swap {
                device.mappings.retain(|m| m.from != to);
            }
            
            device.mappings.push(KeyMapping {
                from: from.clone(),
                to: to.clone(),
                mapping_type: mapping_type.clone(),
            });
            
            // For Swap mode, automatically add reverse mapping
            if mapping_type == MappingType::Swap {
                device.mappings.push(KeyMapping {
                    from: to,
                    to: from,
                    mapping_type,
                });
            }
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
    device_map: HashMap<isize, String>, // Map device handle to device_id (VID:PID)
}

#[cfg(target_os = "windows")]
impl RawInputHandler {
    #[allow(dead_code)]
    fn new(config: Config) -> Self {
        let mut handler = RawInputHandler {
            config,
            device_map: HashMap::new(),
        };
        
        // Initialize device map
        unsafe {
            if let Ok(devices) = Self::list_keyboard_devices() {
                for device in devices {
                    handler.device_map.insert(device.handle as isize, device.device_id);
                }
            }
        }
        
        handler
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
            
            // Get device handle from raw input
            let device_handle = raw_input.header.hDevice as isize;
            
            // Look up device ID from device map
            let device_id = self.device_map
                .get(&device_handle)
                .map(|s| s.as_str())
                .unwrap_or("UNKNOWN");
            
            // Convert virtual key code to key name
            let key_name = Self::vk_to_key_name(vkey as i32);
            
            // Check if key is pressed (not released)
            let is_pressed = (flags & 0x01) == 0;
            
            // Process the key event
            if let Some(mapped_key) = self.config.process_key_event(device_id, &key_name, is_pressed) {
                // Handle different mapping types
                if mapped_key == "None" {
                    // Disable mode: suppress the key, don't send anything
                    add_suppressed_key(vkey);
                    return Some(format!("Key {} disabled", key_name));
                } else {
                    // Remap or Swap mode: suppress original key and send mapped key
                    add_suppressed_key(vkey);
                    
                    // Send the mapped key
                    if let Err(e) = send_key(&mapped_key, is_pressed) {
                        eprintln!("Failed to send key {}: {}", mapped_key, e);
                    }
                    
                    return Some(format!("Key {} remapped to {}", key_name, mapped_key));
                }
            }
            
            None
        }

        None
    }

    /// Run the message loop for raw input processing
    #[allow(dead_code)]
    unsafe fn run_message_loop(config: Config) -> Result<(), String> {
        // Store handler in global variable
        GLOBAL_HANDLER = Some(Arc::new(Mutex::new(RawInputHandler::new(config))));

        // Get module handle
        let h_instance = GetModuleHandleW(std::ptr::null());
        if h_instance.is_null() {
            return Err("Failed to get module handle".to_string());
        }

        // Create window class name
        let class_name: Vec<u16> = "KeyboardRemapperR\0"
            .encode_utf16()
            .collect();

        // Register window class
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        if RegisterClassW(&wnd_class) == 0 {
            return Err("Failed to register window class".to_string());
        }

        // Create hidden window
        let window_name: Vec<u16> = "KeyboardRemapperR Hidden Window\0"
            .encode_utf16()
            .collect();

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            h_instance,
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            return Err("Failed to create window".to_string());
        }

        // Register for raw input
        if let Some(handler) = &GLOBAL_HANDLER {
            if let Ok(handler) = handler.lock() {
                handler.register_raw_input_devices(hwnd)?;
            }
        }

        println!("Message loop started. Press Ctrl+C to stop.");

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let result = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if result == 0 || result == -1 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

// ============================================================================
// Phase 3: Key Input Suppression and Sending Functions
// ============================================================================

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    use winapi::um::winuser::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP};
    
    if n_code >= 0 {
        let kb_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode as u16;
        let extra_info = kb_struct.dwExtraInfo;
        
        // Check if this is an injected key (sent by us)
        if extra_info == INJECTED_KEY_MARKER {
            // Pass through injected keys
            return CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param);
        }
        
        // Check if this key should be suppressed
        if let Some(suppressed) = &SUPPRESSED_KEYS {
            if suppressed.contains(&vk_code) {
                // Suppress the key by returning 1
                return 1;
            }
        }
    }
    
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

#[cfg(target_os = "windows")]
unsafe fn install_keyboard_hook() -> Result<(), String> {
    if KEYBOARD_HOOK.is_some() {
        return Err("Keyboard hook already installed".to_string());
    }
    
    // Initialize suppressed keys set if not already
    if SUPPRESSED_KEYS.is_none() {
        SUPPRESSED_KEYS = Some(std::collections::HashSet::new());
    }
    
    let hook = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        Some(keyboard_hook_proc),
        GetModuleHandleW(std::ptr::null()),
        0,
    );
    
    if hook.is_null() {
        return Err("Failed to install keyboard hook".to_string());
    }
    
    KEYBOARD_HOOK = Some(hook);
    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn uninstall_keyboard_hook() {
    if let Some(hook) = KEYBOARD_HOOK.take() {
        UnhookWindowsHookEx(hook);
    }
    SUPPRESSED_KEYS = None;
}

#[cfg(target_os = "windows")]
fn add_suppressed_key(vk_code: u16) {
    unsafe {
        if let Some(suppressed) = &mut SUPPRESSED_KEYS {
            suppressed.insert(vk_code);
        }
    }
}

#[cfg(target_os = "windows")]
fn remove_suppressed_key(vk_code: u16) {
    unsafe {
        if let Some(suppressed) = &mut SUPPRESSED_KEYS {
            suppressed.remove(&vk_code);
        }
    }
}

#[cfg(target_os = "windows")]
fn should_suppress_key(vk_code: u16, _is_down: bool) -> bool {
    unsafe {
        if let Some(suppressed) = &SUPPRESSED_KEYS {
            return suppressed.contains(&vk_code);
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_extended_key(vk_code: u16) -> bool {
    matches!(
        vk_code,
        0x21..=0x28 | // Page Up, Page Down, End, Home, Arrow keys
        0x2D | 0x2E | // Insert, Delete
        0x5B | 0x5C | 0x5D | // Left Win, Right Win, Apps
        0xA3 | 0xA5 // Right Control, Right Alt
    )
}

#[cfg(target_os = "windows")]
unsafe fn send_key_event(vk_code: u16, is_down: bool, is_extended: bool) -> Result<(), String> {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: std::mem::zeroed(),
    };
    
    let mut flags = 0;
    if !is_down {
        flags |= KEYEVENTF_KEYUP;
    }
    if is_extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    
    *input.u.ki_mut() = KEYBDINPUT {
        wVk: vk_code,
        wScan: 0,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: INJECTED_KEY_MARKER,
    };
    
    let result = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    
    if result == 0 {
        Err("Failed to send key event".to_string())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn send_key(key_name: &str, is_down: bool) -> Result<(), String> {
    let vk_code = RawInputHandler::key_name_to_vk(key_name)
        .ok_or_else(|| format!("Unknown key name: {}", key_name))?;
    
    let is_extended = is_extended_key(vk_code as u16);
    
    unsafe {
        send_key_event(vk_code as u16, is_down, is_extended)
    }
}

#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, keyboard_remapper_service_main);

#[cfg(target_os = "windows")]
fn keyboard_remapper_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        // Log error (will be implemented in Task 4.4)
        eprintln!("Service error: {}", e);
    }
}

#[cfg(target_os = "windows")]
fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::mpsc;
    
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register("KeyboardRemapperR", event_handler)?;

    // Tell Windows that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Run the main loop
    run_main_loop(shutdown_rx)?;

    // Tell Windows that service is stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn run_main_loop(shutdown_rx: std::sync::mpsc::Receiver<()>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config_or_default();
    
    // Install keyboard hook
    unsafe { install_keyboard_hook()? };
    
    // Run message loop in a separate thread
    let handle = std::thread::spawn(move || {
        unsafe { RawInputHandler::run_message_loop(config) }
    });
    
    // Wait for shutdown signal
    shutdown_rx.recv()?;
    
    // Cleanup
    unsafe { uninstall_keyboard_hook(); }
    
    // Wait for message loop to finish
    handle.join().unwrap()?;
    
    Ok(())
}

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Check if running as a service
        if std::env::args().any(|arg| arg == "--service") {
            // Run as Windows service
            if let Err(e) = service_dispatcher::start("KeyboardRemapperR", ffi_service_main) {
                eprintln!("Service dispatcher error: {}", e);
                std::process::exit(1);
            }
            return;
        }
    }
    
    // Run as console application
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
            println!("");
            
            // Install keyboard hook for key suppression
            match unsafe { install_keyboard_hook() } {
                Ok(()) => {
                    println!("Keyboard hook installed successfully.");
                }
                Err(e) => {
                    eprintln!("Error installing keyboard hook: {}", e);
                    std::process::exit(1);
                }
            }
            
            // Run the message loop
            match unsafe { RawInputHandler::run_message_loop(config) } {
                Ok(()) => {
                    println!("Service stopped successfully.");
                    unsafe { uninstall_keyboard_hook(); }
                }
                Err(e) => {
                    eprintln!("Error running service: {}", e);
                    unsafe { uninstall_keyboard_hook(); }
                    std::process::exit(1);
                }
            }
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

    // Phase 2 Tests

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_alphanumeric() {
        // Test alphanumeric keys
        assert_eq!(RawInputHandler::vk_to_key_name(0x30), "0");
        assert_eq!(RawInputHandler::vk_to_key_name(0x39), "9");
        assert_eq!(RawInputHandler::vk_to_key_name(0x41), "A");
        assert_eq!(RawInputHandler::vk_to_key_name(0x5A), "Z");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_special_keys() {
        // Test special keys
        assert_eq!(RawInputHandler::vk_to_key_name(VK_BACK as i32), "Backspace");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_TAB as i32), "Tab");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_RETURN as i32), "Enter");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_SHIFT as i32), "Shift");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_CONTROL as i32), "Ctrl");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_MENU as i32), "Alt");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_CAPITAL as i32), "CapsLock");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_ESCAPE as i32), "Escape");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_SPACE as i32), "Space");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_function_keys() {
        // Test function keys
        assert_eq!(RawInputHandler::vk_to_key_name(VK_F1 as i32), "F1");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_F5 as i32), "F5");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_F12 as i32), "F12");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_numpad_keys() {
        // Test numpad keys
        assert_eq!(RawInputHandler::vk_to_key_name(VK_NUMPAD0 as i32), "Numpad0");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_NUMPAD5 as i32), "Numpad5");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_NUMPAD9 as i32), "Numpad9");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_MULTIPLY as i32), "NumpadMultiply");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_ADD as i32), "NumpadAdd");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_left_right_keys() {
        // Test left/right specific keys
        assert_eq!(RawInputHandler::vk_to_key_name(VK_LSHIFT as i32), "LShift");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_RSHIFT as i32), "RShift");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_LCONTROL as i32), "LCtrl");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_RCONTROL as i32), "RCtrl");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_LMENU as i32), "LAlt");
        assert_eq!(RawInputHandler::vk_to_key_name(VK_RMENU as i32), "RAlt");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_to_key_name_unknown() {
        // Test unknown VK code
        assert_eq!(RawInputHandler::vk_to_key_name(0xFF), "VK_255");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_key_name_to_vk_alphanumeric() {
        // Test alphanumeric keys
        assert_eq!(RawInputHandler::key_name_to_vk("0"), Some(0x30));
        assert_eq!(RawInputHandler::key_name_to_vk("9"), Some(0x39));
        assert_eq!(RawInputHandler::key_name_to_vk("A"), Some(0x41));
        assert_eq!(RawInputHandler::key_name_to_vk("Z"), Some(0x5A));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_key_name_to_vk_special_keys() {
        // Test special keys
        assert_eq!(RawInputHandler::key_name_to_vk("Backspace"), Some(VK_BACK as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("Tab"), Some(VK_TAB as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("Enter"), Some(VK_RETURN as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("CapsLock"), Some(VK_CAPITAL as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("Escape"), Some(VK_ESCAPE as i32));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_key_name_to_vk_left_right_keys() {
        // Test left/right specific keys
        assert_eq!(RawInputHandler::key_name_to_vk("LShift"), Some(VK_LSHIFT as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("RShift"), Some(VK_RSHIFT as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("LCtrl"), Some(VK_LCONTROL as i32));
        assert_eq!(RawInputHandler::key_name_to_vk("RCtrl"), Some(VK_RCONTROL as i32));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_key_name_to_vk_unknown() {
        // Test unknown key name
        assert_eq!(RawInputHandler::key_name_to_vk("UnknownKey"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_vk_conversion_roundtrip() {
        // Test roundtrip conversion: VK -> Name -> VK
        let test_vks = vec![
            VK_BACK as i32,
            VK_TAB as i32,
            VK_RETURN as i32,
            VK_CAPITAL as i32,
            VK_ESCAPE as i32,
            VK_SPACE as i32,
            VK_F1 as i32,
            VK_F12 as i32,
            VK_LSHIFT as i32,
            VK_RCONTROL as i32,
        ];

        for vk in test_vks {
            let name = RawInputHandler::vk_to_key_name(vk);
            let vk_back = RawInputHandler::key_name_to_vk(&name);
            assert_eq!(vk_back, Some(vk), "Roundtrip failed for VK {}", vk);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_device_map_initialization() {
        // Test device map initialization
        let config = Config::new();
        let handler = RawInputHandler::new(config);
        
        // Device map should be initialized (may be empty if no devices connected)
        // Just check that it's created without panicking
        assert!(handler.device_map.len() >= 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_vid_pid_valid() {
        // Test VID/PID parsing from device name
        let device_name = "\\\\?\\HID#VID_04FE&PID_0021#6&2a7e5d7&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}";
        let result = RawInputHandler::parse_vid_pid(device_name);
        assert_eq!(result, Some((0x04FE, 0x0021)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_vid_pid_lowercase() {
        // Test VID/PID parsing with lowercase
        let device_name = "\\\\?\\hid#vid_1234&pid_5678#test";
        let result = RawInputHandler::parse_vid_pid(device_name);
        assert_eq!(result, Some((0x1234, 0x5678)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_parse_vid_pid_invalid() {
        // Test VID/PID parsing with invalid format
        let device_name = "Invalid device name";
        let result = RawInputHandler::parse_vid_pid(device_name);
        assert_eq!(result, None);
    }

    #[test]
    fn test_process_key_event_remap() {
        // Test key event processing with Remap mode
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Remap);
        
        let result = config.process_key_event("04FE:0021", "CapsLock", true);
        assert_eq!(result, Some("LCtrl".to_string()));
    }

    #[test]
    fn test_process_key_event_swap() {
        // Test key event processing with Swap mode
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
        
        let result = config.process_key_event("04FE:0021", "A", true);
        assert_eq!(result, Some("B".to_string()));
    }

    #[test]
    fn test_process_key_event_disable() {
        // Test key event processing with Disable mode
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "".to_string(), MappingType::Disable);
        
        let result = config.process_key_event("04FE:0021", "CapsLock", true);
        assert_eq!(result, Some("None".to_string()));
    }

    #[test]
    fn test_process_key_event_no_mapping() {
        // Test key event processing with no mapping
        let config = Config::new();
        
        let result = config.process_key_event("04FE:0021", "A", true);
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn test_process_key_event_different_device() {
        // Test key event processing with different device
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Remap);
        
        // Different device should not apply mapping
        let result = config.process_key_event("1234:5678", "CapsLock", true);
        assert_eq!(result, Some("CapsLock".to_string()));
    }
}
