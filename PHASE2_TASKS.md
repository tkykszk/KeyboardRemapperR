# Phase 2: キーボード入力フックの実装 - 詳細タスク表

**作成日**: 2026年1月14日  
**前提条件**: Phase 1 完了（デバイス検出とID抽出）  
**目標**: Raw Input メッセージを受信し、キー入力を処理する基盤を構築

---

## 📋 Phase 2 概要

Phase 2 では、Windows のメッセージループを実装し、キーボード入力イベントをリアルタイムで受信・処理する機能を構築します。これにより、実際のキー入力をデバイス別に識別し、マッピング設定に基づいて処理する準備が整います。

### 主要な実装項目

| タスク | 内容 | 見積もり | 依存関係 |
|--------|------|----------|----------|
| Task 2.1 | ウィンドウメッセージループの実装 | 6-8時間 | なし |
| Task 2.2 | Raw Input イベント処理の統合 | 4-5時間 | Task 2.1, Phase 1 |
| Task 2.3 | 仮想キーコード変換テーブルの実装 | 3-4時間 | なし |
| Task 2.4 | キー名 → VK コード逆変換の実装 | 2-3時間 | Task 2.3 |
| **合計** | - | **15-20時間** | - |

---

## Task 2.1: ウィンドウメッセージループの実装

### 目的

Raw Input メッセージ（`WM_INPUT`）を受信するための非表示ウィンドウとメッセージループを実装します。これは Windows アプリケーションの基本構造であり、キーボード入力イベントを受け取るために必須です。

### 実装内容

#### 2.1.1: ウィンドウクラスの登録

Windows でウィンドウを作成するには、まずウィンドウクラスを登録する必要があります。`RegisterClassW` API を使用して、ウィンドウプロシージャを含むウィンドウクラスを登録します。

**必要なWinAPI**:
- `RegisterClassW`: ウィンドウクラスを登録
- `WNDCLASSW`: ウィンドウクラス構造体
- `DefWindowProcW`: デフォルトのウィンドウプロシージャ

**実装例**:
```rust
use winapi::um::winuser::{
    RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
};
use winapi::um::libloaderapi::GetModuleHandleW;

unsafe fn register_window_class() -> Result<u16, String> {
    let class_name = wide_string("KeyboardRemapperR");
    
    let wnd_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: GetModuleHandleW(std::ptr::null()),
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    
    let atom = RegisterClassW(&wnd_class);
    if atom == 0 {
        Err("Failed to register window class".to_string())
    } else {
        Ok(atom)
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
```

#### 2.1.2: 非表示ウィンドウの作成

`CreateWindowExW` API を使用して、メッセージを受信するための非表示ウィンドウを作成します。このウィンドウは画面に表示されませんが、Raw Input メッセージを受信できます。

**必要なWinAPI**:
- `CreateWindowExW`: ウィンドウを作成
- `WS_OVERLAPPEDWINDOW`: ウィンドウスタイル
- `HWND_MESSAGE`: メッセージ専用ウィンドウ

**実装例**:
```rust
use winapi::um::winuser::{
    CreateWindowExW, HWND_MESSAGE, WS_OVERLAPPEDWINDOW,
};

unsafe fn create_message_window(class_name: &[u16]) -> Result<HWND, String> {
    let window_name = wide_string("KeyboardRemapperR Message Window");
    
    let hwnd = CreateWindowExW(
        0,                          // dwExStyle
        class_name.as_ptr(),        // lpClassName
        window_name.as_ptr(),       // lpWindowName
        WS_OVERLAPPEDWINDOW,        // dwStyle
        0, 0, 0, 0,                 // x, y, width, height
        HWND_MESSAGE,               // hWndParent (message-only window)
        std::ptr::null_mut(),       // hMenu
        GetModuleHandleW(std::ptr::null()), // hInstance
        std::ptr::null_mut(),       // lpParam
    );
    
    if hwnd.is_null() {
        Err("Failed to create window".to_string())
    } else {
        Ok(hwnd)
    }
}
```

#### 2.1.3: ウィンドウプロシージャの実装

ウィンドウプロシージャは、ウィンドウに送られるメッセージを処理するコールバック関数です。`WM_INPUT` メッセージを処理し、その他のメッセージは `DefWindowProcW` に転送します。

**必要なWinAPI**:
- `WM_INPUT`: Raw Input メッセージ
- `WM_DESTROY`: ウィンドウ破棄メッセージ
- `DefWindowProcW`: デフォルトのメッセージ処理

**実装例**:
```rust
use winapi::um::winuser::{
    DefWindowProcW, PostQuitMessage, WM_DESTROY, WM_INPUT,
};
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT};

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            // Raw Input メッセージを処理
            // TODO: RawInputHandler::process_raw_input を呼び出す
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
```

#### 2.1.4: メッセージループの実装

`GetMessage` と `DispatchMessage` を使用して、メッセージループを実装します。このループは、ウィンドウにメッセージが送られるたびに実行され、`WM_QUIT` メッセージを受信するまで継続します。

**必要なWinAPI**:
- `GetMessageW`: メッセージを取得
- `TranslateMessage`: キーボードメッセージを変換
- `DispatchMessageW`: メッセージをウィンドウプロシージャに送信

**実装例**:
```rust
use winapi::um::winuser::{
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
};

unsafe fn run_message_loop() {
    let mut msg: MSG = std::mem::zeroed();
    
    loop {
        let result = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
        
        if result == 0 {
            // WM_QUIT received
            break;
        } else if result == -1 {
            // Error
            eprintln!("GetMessage error");
            break;
        }
        
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
```

#### 2.1.5: WindowMessageLoop 構造体の作成

上記の機能を統合する `WindowMessageLoop` 構造体を作成します。この構造体は、ウィンドウの作成、Raw Input デバイスの登録、メッセージループの実行を管理します。

**実装例**:
```rust
#[cfg(target_os = "windows")]
struct WindowMessageLoop {
    hwnd: HWND,
    handler: RawInputHandler,
}

#[cfg(target_os = "windows")]
impl WindowMessageLoop {
    unsafe fn new(config: Config) -> Result<Self, String> {
        // ウィンドウクラスを登録
        register_window_class()?;
        
        // ウィンドウを作成
        let class_name = wide_string("KeyboardRemapperR");
        let hwnd = create_message_window(&class_name)?;
        
        // RawInputHandler を作成
        let mut handler = RawInputHandler::new(config);
        
        // Raw Input デバイスを登録
        handler.register_raw_input_devices(hwnd)?;
        
        Ok(WindowMessageLoop { hwnd, handler })
    }
    
    unsafe fn run(&mut self) {
        run_message_loop();
    }
}
```

### 成果物

- `register_window_class()` 関数
- `create_message_window()` 関数
- `window_proc()` コールバック関数
- `run_message_loop()` 関数
- `WindowMessageLoop` 構造体

### テスト方法

1. ウィンドウが正常に作成されることを確認
2. メッセージループが起動することを確認
3. Ctrl+C で終了できることを確認

### 見積もり時間

**6-8時間**

---

## Task 2.2: Raw Input イベント処理の統合

### 目的

既存の `RawInputHandler::process_raw_input` メソッドをメッセージループに統合し、Phase 1 で実装したデバイスハンドルから VID/PID への変換機能を活用します。

### 実装内容

#### 2.2.1: グローバル状態の管理

ウィンドウプロシージャは静的関数であるため、`RawInputHandler` インスタンスにアクセスするためのグローバル状態管理が必要です。`lazy_static` または `once_cell` クレートを使用して、スレッドセーフなグローバル変数を作成します。

**依存関係の追加**:
```toml
[dependencies]
once_cell = "1.19"
```

**実装例**:
```rust
use once_cell::sync::Mutex;
use std::sync::Arc;

static HANDLER: once_cell::sync::OnceCell<Arc<Mutex<RawInputHandler>>> = 
    once_cell::sync::OnceCell::new();

fn set_global_handler(handler: RawInputHandler) {
    HANDLER.set(Arc::new(Mutex::new(handler))).ok();
}

fn get_global_handler() -> Option<Arc<Mutex<RawInputHandler>>> {
    HANDLER.get().cloned()
}
```

#### 2.2.2: デバイスレジストリの実装

デバイスハンドルから VID/PID を取得するための `DeviceRegistry` を実装します。これは、Phase 1 で取得したデバイス情報をキャッシュし、高速に検索できるようにします。

**実装例**:
```rust
use std::collections::HashMap;

#[cfg(target_os = "windows")]
struct DeviceRegistry {
    devices: HashMap<isize, KeyboardDeviceInfo>, // HANDLE -> DeviceInfo
}

#[cfg(target_os = "windows")]
impl DeviceRegistry {
    fn new() -> Self {
        DeviceRegistry {
            devices: HashMap::new(),
        }
    }
    
    unsafe fn refresh(&mut self) -> Result<(), String> {
        self.devices.clear();
        
        let devices = RawInputHandler::list_keyboard_devices()?;
        for device in devices {
            self.devices.insert(device.handle as isize, device);
        }
        
        Ok(())
    }
    
    fn get_device_id(&self, handle: HANDLE) -> Option<String> {
        self.devices.get(&(handle as isize))
            .map(|d| d.device_id.clone())
    }
}
```

#### 2.2.3: RawInputHandler の拡張

`RawInputHandler` に `DeviceRegistry` を追加し、デバイスハンドルから VID/PID を取得できるようにします。

**実装例**:
```rust
#[cfg(target_os = "windows")]
struct RawInputHandler {
    config: Config,
    device_registry: DeviceRegistry,
}

#[cfg(target_os = "windows")]
impl RawInputHandler {
    fn new(config: Config) -> Self {
        let mut handler = RawInputHandler {
            config,
            device_registry: DeviceRegistry::new(),
        };
        
        // デバイスレジストリを初期化
        unsafe {
            handler.device_registry.refresh().ok();
        }
        
        handler
    }
}
```

#### 2.2.4: process_raw_input の更新

`process_raw_input` メソッドを更新し、実際のデバイスハンドルから VID/PID を取得するようにします。

**実装例**:
```rust
unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
    let mut size: UINT = 0;
    
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
    
    if raw_input.header.dwType == RIM_TYPEKEYBOARD {
        let keyboard = raw_input.data.keyboard();
        let vkey = keyboard.VKey;
        let flags = keyboard.Flags;
        
        // 実際のデバイスハンドルから VID/PID を取得
        let device_handle = raw_input.header.hDevice;
        let device_id = self.device_registry.get_device_id(device_handle)
            .unwrap_or_else(|| "0000:0000".to_string());
        
        // VK コードをキー名に変換（Task 2.3 で実装）
        let key_name = format!("VK_{}", vkey);
        
        let is_pressed = (flags & 0x01) == 0;
        
        // マッピングを適用
        return self.config.process_key_event(&device_id, &key_name, is_pressed);
    }

    None
}
```

#### 2.2.5: ウィンドウプロシージャの更新

ウィンドウプロシージャで `process_raw_input` を呼び出すように更新します。

**実装例**:
```rust
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            if let Some(handler) = get_global_handler() {
                if let Ok(mut h) = handler.lock() {
                    if let Some(result) = h.process_raw_input(lparam) {
                        println!("Processed key: {}", result);
                    }
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
```

### 成果物

- `DeviceRegistry` 構造体
- 更新された `RawInputHandler`
- 更新された `process_raw_input` メソッド
- グローバル状態管理機能

### テスト方法

1. キーボードを押したときに `WM_INPUT` メッセージが受信されることを確認
2. デバイスハンドルから正しい VID/PID が取得されることを確認
3. 複数のキーボードを接続して、デバイス別に識別されることを確認

### 見積もり時間

**4-5時間**

---

## Task 2.3: 仮想キーコード変換テーブルの実装

### 目的

Windows の仮想キーコード（VK コード）を人間が読めるキー名に変換するテーブルを実装します。これにより、設定ファイルで `CapsLock` や `A` などの名前を使用できるようになります。

### 実装内容

#### 2.3.1: VK コード定数の定義

Windows の主要な VK コードを定数として定義します。

**実装例**:
```rust
// 文字キー
const VK_A: u16 = 0x41;
const VK_B: u16 = 0x42;
// ... (Z まで)

// 数字キー
const VK_0: u16 = 0x30;
const VK_1: u16 = 0x31;
// ... (9 まで)

// 特殊キー
const VK_BACK: u16 = 0x08;
const VK_TAB: u16 = 0x09;
const VK_RETURN: u16 = 0x0D;
const VK_SHIFT: u16 = 0x10;
const VK_CONTROL: u16 = 0x11;
const VK_MENU: u16 = 0x12;  // Alt
const VK_CAPITAL: u16 = 0x14;  // CapsLock
const VK_ESCAPE: u16 = 0x1B;
const VK_SPACE: u16 = 0x20;

// ファンクションキー
const VK_F1: u16 = 0x70;
const VK_F2: u16 = 0x71;
// ... (F24 まで)

// 左右の修飾キー
const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;
const VK_LMENU: u16 = 0xA4;  // Left Alt
const VK_RMENU: u16 = 0xA5;  // Right Alt
```

#### 2.3.2: VK コード → キー名変換関数

VK コードをキー名に変換する関数を実装します。

**実装例**:
```rust
fn vk_to_key_name(vk: u16) -> String {
    match vk {
        // 文字キー (A-Z)
        0x41..=0x5A => {
            let c = (vk - 0x41 + b'A') as char;
            c.to_string()
        }
        // 数字キー (0-9)
        0x30..=0x39 => {
            let c = (vk - 0x30 + b'0') as char;
            c.to_string()
        }
        // 特殊キー
        VK_BACK => "Backspace".to_string(),
        VK_TAB => "Tab".to_string(),
        VK_RETURN => "Enter".to_string(),
        VK_SHIFT => "Shift".to_string(),
        VK_CONTROL => "Ctrl".to_string(),
        VK_MENU => "Alt".to_string(),
        VK_CAPITAL => "CapsLock".to_string(),
        VK_ESCAPE => "Escape".to_string(),
        VK_SPACE => "Space".to_string(),
        // 左右の修飾キー
        VK_LSHIFT => "LShift".to_string(),
        VK_RSHIFT => "RShift".to_string(),
        VK_LCONTROL => "LCtrl".to_string(),
        VK_RCONTROL => "RCtrl".to_string(),
        VK_LMENU => "LAlt".to_string(),
        VK_RMENU => "RAlt".to_string(),
        // ファンクションキー (F1-F24)
        0x70..=0x87 => {
            format!("F{}", vk - 0x70 + 1)
        }
        // JIS キーボード固有キー
        0xF3 => "Katakana".to_string(),
        0xF4 => "Hiragana".to_string(),
        0x1C => "Convert".to_string(),      // 変換
        0x1D => "NonConvert".to_string(),   // 無変換
        // その他
        _ => format!("VK_{:02X}", vk),
    }
}
```

#### 2.3.3: テストケースの追加

VK コード変換のテストケースを追加します。

**実装例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vk_to_key_name() {
        assert_eq!(vk_to_key_name(0x41), "A");
        assert_eq!(vk_to_key_name(0x5A), "Z");
        assert_eq!(vk_to_key_name(0x30), "0");
        assert_eq!(vk_to_key_name(0x39), "9");
        assert_eq!(vk_to_key_name(VK_CAPITAL), "CapsLock");
        assert_eq!(vk_to_key_name(VK_LCONTROL), "LCtrl");
        assert_eq!(vk_to_key_name(VK_F1), "F1");
        assert_eq!(vk_to_key_name(VK_F12), "F12");
    }
}
```

### 成果物

- VK コード定数の定義
- `vk_to_key_name()` 関数
- テストケース

### テスト方法

1. ユニットテストを実行
2. 実際のキー入力で正しい名前が表示されることを確認

### 見積もり時間

**3-4時間**

---

## Task 2.4: キー名 → VK コード逆変換の実装

### 目的

設定ファイルのキー名（例: `CapsLock`、`A`）を VK コードに変換する機能を実装します。これにより、ユーザーが設定したマッピングを実際のキー入力に適用できるようになります。

### 実装内容

#### 2.4.1: キー名 → VK コード変換関数

キー名を VK コードに変換する関数を実装します。大文字小文字を正規化し、エイリアスにも対応します。

**実装例**:
```rust
fn key_name_to_vk(name: &str) -> Option<u16> {
    let normalized = name.to_uppercase();
    
    match normalized.as_str() {
        // 文字キー (A-Z)
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
            let c = s.chars().next().unwrap();
            Some((c as u16) - ('A' as u16) + 0x41)
        }
        // 数字キー (0-9)
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
            let c = s.chars().next().unwrap();
            Some((c as u16) - ('0' as u16) + 0x30)
        }
        // 特殊キー
        "BACKSPACE" | "BACK" => Some(VK_BACK),
        "TAB" => Some(VK_TAB),
        "ENTER" | "RETURN" => Some(VK_RETURN),
        "SHIFT" => Some(VK_SHIFT),
        "CTRL" | "CONTROL" => Some(VK_CONTROL),
        "ALT" | "MENU" => Some(VK_MENU),
        "CAPSLOCK" | "CAPS" => Some(VK_CAPITAL),
        "ESCAPE" | "ESC" => Some(VK_ESCAPE),
        "SPACE" => Some(VK_SPACE),
        // 左右の修飾キー
        "LSHIFT" => Some(VK_LSHIFT),
        "RSHIFT" => Some(VK_RSHIFT),
        "LCTRL" | "LCONTROL" => Some(VK_LCONTROL),
        "RCTRL" | "RCONTROL" => Some(VK_RCONTROL),
        "LALT" | "LMENU" => Some(VK_LMENU),
        "RALT" | "RMENU" => Some(VK_RMENU),
        // ファンクションキー (F1-F24)
        s if s.starts_with('F') => {
            let num_str = &s[1..];
            if let Ok(num) = num_str.parse::<u16>() {
                if num >= 1 && num <= 24 {
                    return Some(0x70 + num - 1);
                }
            }
            None
        }
        // JIS キーボード固有キー
        "KATAKANA" => Some(0xF3),
        "HIRAGANA" => Some(0xF4),
        "CONVERT" | "変換" => Some(0x1C),
        "NONCONVERT" | "無変換" => Some(0x1D),
        // VK_XX 形式
        s if s.starts_with("VK_") => {
            let hex_str = &s[3..];
            u16::from_str_radix(hex_str, 16).ok()
        }
        _ => None,
    }
}
```

#### 2.4.2: テストケースの追加

キー名変換のテストケースを追加します。

**実装例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_name_to_vk() {
        assert_eq!(key_name_to_vk("A"), Some(0x41));
        assert_eq!(key_name_to_vk("a"), Some(0x41));
        assert_eq!(key_name_to_vk("Z"), Some(0x5A));
        assert_eq!(key_name_to_vk("0"), Some(0x30));
        assert_eq!(key_name_to_vk("9"), Some(0x39));
        assert_eq!(key_name_to_vk("CapsLock"), Some(VK_CAPITAL));
        assert_eq!(key_name_to_vk("CAPSLOCK"), Some(VK_CAPITAL));
        assert_eq!(key_name_to_vk("Ctrl"), Some(VK_CONTROL));
        assert_eq!(key_name_to_vk("LCtrl"), Some(VK_LCONTROL));
        assert_eq!(key_name_to_vk("F1"), Some(VK_F1));
        assert_eq!(key_name_to_vk("F12"), Some(0x7B));
        assert_eq!(key_name_to_vk("VK_14"), Some(VK_CAPITAL));
        assert_eq!(key_name_to_vk("Invalid"), None);
    }
}
```

#### 2.4.3: process_key_event の更新

`Config::process_key_event` メソッドを更新し、キー名を VK コードに変換してから処理するようにします。

**実装例**:
```rust
impl Config {
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
```

### 成果物

- `key_name_to_vk()` 関数
- テストケース
- 更新された `process_key_event` メソッド

### テスト方法

1. ユニットテストを実行
2. 設定ファイルでキー名を使用してマッピングを設定
3. 実際のキー入力で正しく変換されることを確認

### 見積もり時間

**2-3時間**

---

## Phase 2 統合テスト

### テストシナリオ

#### シナリオ1: 基本的なキー入力の受信

1. プログラムを起動
2. キーボードのキーを押す
3. コンソールにキー名が表示されることを確認

**期待される出力**:
```
Processed key: A
Processed key: CapsLock
Processed key: Enter
```

#### シナリオ2: デバイス別の識別

1. 複数のキーボードを接続
2. プログラムを起動
3. 各キーボードのキーを押す
4. デバイスIDが正しく識別されることを確認

**期待される出力**:
```
Device: 04FE:0021, Key: A
Device: 046D:C52B, Key: A
```

#### シナリオ3: マッピングの適用

1. CapsLock → LCtrl のマッピングを設定
2. プログラムを起動
3. CapsLock キーを押す
4. LCtrl として処理されることを確認

**期待される出力**:
```
Processed key: LCtrl (mapped from CapsLock)
```

### テストコマンド

```bash
# ビルド
cargo build

# 実行
cargo run -- start

# 別のターミナルで設定
cargo run -- set 04FE:0021 CapsLock LCtrl
```

---

## Phase 2 完了基準

以下のすべての項目が完了した時点で、Phase 2 完了とします。

### 機能要件

- ✅ 非表示ウィンドウが作成される
- ✅ メッセージループが正常に動作する
- ✅ `WM_INPUT` メッセージが受信される
- ✅ デバイスハンドルから VID/PID が取得される
- ✅ VK コードがキー名に変換される
- ✅ キー名が VK コードに変換される

### テスト要件

- ✅ すべてのユニットテストが通過
- ✅ 実機でのキー入力が正しく受信される
- ✅ 複数のキーボードが識別される

### ドキュメント要件

- ✅ コード内にコメントが適切に記載されている
- ✅ Phase 2 完了レポートが作成されている

---

## 次のステップ（Phase 3）

Phase 2 完了後、Phase 3「キー入力送信の実装」に進みます。

### Phase 3 の主要タスク

**Task 3.1: キー入力抑制機能の実装**

Low-level keyboard hook（`SetWindowsHookEx` with `WH_KEYBOARD_LL`）を実装し、リマップ元のキー入力を無効化します。

**Task 3.2: キー入力送信機能の実装**

`SendInput` API を使用して、リマップ先のキーを送信します。`KEYBDINPUT` 構造体を構築し、キー押下/解放イベントを生成します。

**Task 3.3: Swap モードの実装**

2つのキーを相互に入れ替える機能を実装します。双方向マッピングの管理と循環参照の検出を行います。

**Task 3.4: Disable モードの実装**

特定のキーを無効化する機能を実装します。キー入力の抑制のみを行い、送信は行いません。

---

## 必要な追加依存関係

```toml
[dependencies]
# 既存
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Phase 2 で追加
once_cell = "1.19"  # グローバル状態管理

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winuser",
    "windef",
    "minwindef",
    "hidusage",
    "hidsdi",
    "libloaderapi",  # 新規: GetModuleHandleW
] }
```

---

## 実装時の注意事項

### 1. スレッドセーフティ

メッセージループは単一スレッドで動作しますが、将来的な拡張を考慮して、グローバル状態は `Mutex` で保護します。

### 2. エラーハンドリング

Windows API のエラーは `GetLastError` で取得できますが、現時点では簡易的なエラーメッセージで対応します。Phase 5 でログ機能を実装する際に改善します。

### 3. メモリ管理

`Vec<u8>` を使用して Raw Input データのバッファを確保します。バッファサイズは動的に取得するため、メモリリークの心配はありません。

### 4. パフォーマンス

キー入力イベントは高頻度で発生するため、`process_raw_input` メソッドは可能な限り高速に実行される必要があります。重い処理は避け、必要に応じて別スレッドで処理します。

---

## まとめ

Phase 2 では、Windows のメッセージループを実装し、キーボード入力イベントをリアルタイムで受信・処理する基盤を構築します。これにより、実際のキー入力をデバイス別に識別し、マッピング設定に基づいて処理する準備が整います。

**見積もり時間**: 15-20時間  
**実装順序**: Task 2.1 → Task 2.3 → Task 2.4 → Task 2.2  
**次のマイルストーン**: Phase 3 完了 → v0.1.0-beta1 リリース

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: 1.0
