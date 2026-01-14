# Phase 3: キー入力送信の実装 - 詳細タスク表

**作成日**: 2026年1月14日  
**前提条件**: Phase 1-2 完了（デバイス検出、メッセージループ）  
**目標**: 実際のキーリマップ機能を実現し、v0.1.0-beta1 をリリース

---

## 📋 Phase 3 概要

Phase 3 では、キーボード入力を抑制し、リマップ先のキーを送信する機能を実装します。これにより、KeyboardRemapperR の中核機能である「デバイス別キーリマップ」が完全に動作するようになります。

### 主要な実装項目

| タスク | 内容 | 見積もり | 依存関係 |
|--------|------|----------|----------|
| Task 3.1 | キー入力抑制機能の実装 | 6-8時間 | Phase 2 |
| Task 3.2 | キー入力送信機能の実装 | 5-6時間 | Task 3.1 |
| Task 3.3 | Swap モードの実装 | 3-4時間 | Task 3.2 |
| Task 3.4 | Disable モードの実装 | 2-3時間 | Task 3.1 |
| **合計** | - | **16-21時間** | - |

---

## Task 3.1: キー入力抑制機能の実装

### 目的

リマップ元のキー入力を無効化する機能を実装します。Low-level keyboard hook を使用して、キーボード入力をシステムレベルで捕捉し、必要に応じて抑制します。

### 背景知識

Windows には2種類のキーボードフック機構があります。

**Raw Input vs Low-level Keyboard Hook**:

| 機能 | Raw Input | Low-level Keyboard Hook |
|------|-----------|------------------------|
| 用途 | デバイス別の入力検出 | システム全体の入力制御 |
| 入力抑制 | ❌ 不可能 | ✅ 可能 |
| デバイス識別 | ✅ 可能 | ❌ 不可能 |
| 管理者権限 | 不要 | 必要 |

KeyboardRemapperR では、**両方を組み合わせて使用**します。Raw Input でデバイスを識別し、Low-level Keyboard Hook で入力を抑制します。

### 実装内容

#### 3.1.1: Low-level Keyboard Hook の基本実装

`SetWindowsHookEx` API を使用して、Low-level keyboard hook を設定します。

**必要なWinAPI**:
- `SetWindowsHookEx`: フックを設定
- `UnhookWindowsHookEx`: フックを解除
- `CallNextHookEx`: 次のフックに処理を渡す
- `WH_KEYBOARD_LL`: Low-level keyboard hook タイプ
- `KBDLLHOOKSTRUCT`: キーボードフック情報構造体

**実装例**:
```rust
use winapi::um::winuser::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, KBDLLHOOKSTRUCT, HC_ACTION,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};
use winapi::shared::windef::HHOOK;
use winapi::shared::minwindef::{WPARAM, LPARAM, LRESULT};

static mut KEYBOARD_HOOK: Option<HHOOK> = None;

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == HC_ACTION {
        let kb_struct = &*(lparam as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode as u16;
        let is_key_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
        
        // キーを抑制するかどうかを判定
        if should_suppress_key(vk_code, is_key_down) {
            return 1; // 1 を返すとキー入力が抑制される
        }
    }
    
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

unsafe fn install_keyboard_hook() -> Result<(), String> {
    let hook = SetWindowsHookExW(
        WH_KEYBOARD_LL,
        Some(keyboard_hook_proc),
        std::ptr::null_mut(),
        0,
    );
    
    if hook.is_null() {
        Err("Failed to install keyboard hook".to_string())
    } else {
        KEYBOARD_HOOK = Some(hook);
        Ok(())
    }
}

unsafe fn uninstall_keyboard_hook() {
    if let Some(hook) = KEYBOARD_HOOK {
        UnhookWindowsHookEx(hook);
        KEYBOARD_HOOK = None;
    }
}
```

#### 3.1.2: キー抑制判定ロジックの実装

どのキーを抑制するかを判定するロジックを実装します。Raw Input で検出したデバイスとキーの組み合わせに基づいて判定します。

**課題**: Low-level keyboard hook ではデバイスを識別できないため、Raw Input との連携が必要です。

**解決策**: Raw Input で最後に検出されたキーを記録し、Low-level keyboard hook でそのキーを抑制します。

**実装例**:
```rust
use std::sync::Mutex;
use std::collections::HashSet;

// 抑制するキーのセット（VKコード）
static SUPPRESSED_KEYS: once_cell::sync::Lazy<Mutex<HashSet<u16>>> = 
    once_cell::sync::Lazy::new(|| Mutex::new(HashSet::new()));

fn add_suppressed_key(vk: u16) {
    if let Ok(mut keys) = SUPPRESSED_KEYS.lock() {
        keys.insert(vk);
    }
}

fn remove_suppressed_key(vk: u16) {
    if let Ok(mut keys) = SUPPRESSED_KEYS.lock() {
        keys.remove(&vk);
    }
}

fn should_suppress_key(vk: u16, _is_key_down: bool) -> bool {
    if let Ok(keys) = SUPPRESSED_KEYS.lock() {
        keys.contains(&vk)
    } else {
        false
    }
}
```

#### 3.1.3: Raw Input との連携

Raw Input で検出したキーが、マッピング対象である場合、そのキーを抑制リストに追加します。

**実装例**:
```rust
impl RawInputHandler {
    unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
        // ... (既存のコード)
        
        if raw_input.header.dwType == RIM_TYPEKEYBOARD {
            let keyboard = raw_input.data.keyboard();
            let vkey = keyboard.VKey;
            let flags = keyboard.Flags;
            
            let device_handle = raw_input.header.hDevice;
            let device_id = self.device_registry.get_device_id(device_handle)
                .unwrap_or_else(|| "0000:0000".to_string());
            
            let key_name = vk_to_key_name(vkey);
            let is_pressed = (flags & 0x01) == 0;
            
            // マッピングを確認
            if let Some(device) = self.config.devices.iter()
                .find(|d| d.device_id == device_id) {
                if let Some(mapping) = device.mappings.iter()
                    .find(|m| m.from == key_name) {
                    
                    // マッピングが存在する場合、キーを抑制
                    if is_pressed {
                        add_suppressed_key(vkey);
                    } else {
                        remove_suppressed_key(vkey);
                    }
                    
                    // リマップ先のキーを返す
                    return Some(mapping.to.clone());
                }
            }
            
            Some(key_name)
        } else {
            None
        }
    }
}
```

#### 3.1.4: タイミング問題の解決

Raw Input と Low-level keyboard hook の処理順序により、タイミング問題が発生する可能性があります。

**問題**: Low-level keyboard hook が Raw Input よりも先に呼ばれる場合、抑制判定が間に合わない。

**解決策1**: 小さな遅延を導入（非推奨、入力遅延が発生）

**解決策2**: すべてのマッピング対象キーを事前に抑制リストに追加し、Raw Input で処理後に送信（推奨）

**実装例（解決策2）**:
```rust
impl RawInputHandler {
    fn new(config: Config) -> Self {
        let mut handler = RawInputHandler {
            config,
            device_registry: DeviceRegistry::new(),
        };
        
        unsafe {
            handler.device_registry.refresh().ok();
            handler.initialize_suppressed_keys();
        }
        
        handler
    }
    
    fn initialize_suppressed_keys(&self) {
        // すべてのマッピング元キーを抑制リストに追加
        for device in &self.config.devices {
            for mapping in &device.mappings {
                if let Some(vk) = key_name_to_vk(&mapping.from) {
                    add_suppressed_key(vk);
                }
            }
        }
    }
}
```

### 成果物

- `keyboard_hook_proc()` コールバック関数
- `install_keyboard_hook()` 関数
- `uninstall_keyboard_hook()` 関数
- キー抑制判定ロジック
- Raw Input との連携機能

### テスト方法

1. CapsLock キーを抑制リストに追加
2. プログラムを起動
3. CapsLock キーを押す
4. CapsLock が動作しないことを確認（LED が点灯しない）

### 見積もり時間

**6-8時間**

---

## Task 3.2: キー入力送信機能の実装

### 目的

リマップ先のキーを送信する機能を実装します。`SendInput` API を使用して、仮想的なキーボード入力を生成します。

### 実装内容

#### 3.2.1: SendInput API の基本実装

`SendInput` API を使用して、キーボード入力を送信します。

**必要なWinAPI**:
- `SendInput`: 入力イベントを送信
- `INPUT`: 入力イベント構造体
- `KEYBDINPUT`: キーボード入力構造体
- `KEYEVENTF_KEYUP`: キー解放フラグ
- `KEYEVENTF_EXTENDEDKEY`: 拡張キーフラグ

**実装例**:
```rust
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_SCANCODE,
};

unsafe fn send_key_event(vk: u16, is_key_down: bool, is_extended: bool) -> Result<(), String> {
    let mut input: INPUT = std::mem::zeroed();
    input.type_ = INPUT_KEYBOARD;
    
    let mut ki: KEYBDINPUT = std::mem::zeroed();
    ki.wVk = vk;
    ki.wScan = 0;
    ki.dwFlags = if is_key_down { 0 } else { KEYEVENTF_KEYUP };
    
    if is_extended {
        ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
    }
    
    ki.time = 0;
    ki.dwExtraInfo = 0;
    
    *input.u.ki_mut() = ki;
    
    let result = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    
    if result == 0 {
        Err("Failed to send input".to_string())
    } else {
        Ok(())
    }
}
```

#### 3.2.2: 拡張キーの判定

一部のキー（矢印キー、Home、End など）は拡張キーとして扱う必要があります。

**拡張キーのリスト**:
- 矢印キー（Up, Down, Left, Right）
- Home, End, PageUp, PageDown
- Insert, Delete
- 右側の修飾キー（RCtrl, RAlt）
- NumLock, PrintScreen, Pause

**実装例**:
```rust
fn is_extended_key(vk: u16) -> bool {
    matches!(vk,
        0x21..=0x2E | // PageUp, PageDown, End, Home, Arrow keys, Insert, Delete
        0x5B..=0x5C | // Left Windows, Right Windows
        0xA3 |        // Right Control
        0xA5          // Right Alt
    )
}
```

#### 3.2.3: キー送信関数の実装

キー名から VK コードに変換し、キーを送信する関数を実装します。

**実装例**:
```rust
fn send_key(key_name: &str, is_key_down: bool) -> Result<(), String> {
    if let Some(vk) = key_name_to_vk(key_name) {
        let is_extended = is_extended_key(vk);
        unsafe {
            send_key_event(vk, is_key_down, is_extended)
        }
    } else {
        Err(format!("Unknown key name: {}", key_name))
    }
}
```

#### 3.2.4: process_raw_input の更新

Raw Input で検出したキーをリマップし、リマップ先のキーを送信します。

**実装例**:
```rust
impl RawInputHandler {
    unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
        // ... (既存のコード)
        
        if raw_input.header.dwType == RIM_TYPEKEYBOARD {
            let keyboard = raw_input.data.keyboard();
            let vkey = keyboard.VKey;
            let flags = keyboard.Flags;
            
            let device_handle = raw_input.header.hDevice;
            let device_id = self.device_registry.get_device_id(device_handle)
                .unwrap_or_else(|| "0000:0000".to_string());
            
            let key_name = vk_to_key_name(vkey);
            let is_pressed = (flags & 0x01) == 0;
            
            // マッピングを確認
            if let Some(device) = self.config.devices.iter()
                .find(|d| d.device_id == device_id) {
                if let Some(mapping) = device.mappings.iter()
                    .find(|m| m.from == key_name) {
                    
                    match mapping.mapping_type {
                        MappingType::Remap => {
                            // リマップ先のキーを送信
                            send_key(&mapping.to, is_pressed).ok();
                            return Some(format!("{} -> {}", key_name, mapping.to));
                        }
                        MappingType::Swap => {
                            // Swap は Task 3.3 で実装
                            send_key(&mapping.to, is_pressed).ok();
                            return Some(format!("{} <-> {}", key_name, mapping.to));
                        }
                        MappingType::Disable => {
                            // キーを抑制するだけ（送信しない）
                            return Some(format!("{} (disabled)", key_name));
                        }
                    }
                }
            }
            
            Some(key_name)
        } else {
            None
        }
    }
}
```

#### 3.2.5: 無限ループの防止

`SendInput` で送信したキーが再び Raw Input で検出され、無限ループが発生する可能性があります。

**解決策**: `SendInput` で送信する際に `dwExtraInfo` フィールドに特別な値を設定し、Raw Input で検出時にチェックします。

**実装例**:
```rust
const INJECTED_KEY_MARKER: usize = 0x12345678;

unsafe fn send_key_event(vk: u16, is_key_down: bool, is_extended: bool) -> Result<(), String> {
    let mut input: INPUT = std::mem::zeroed();
    input.type_ = INPUT_KEYBOARD;
    
    let mut ki: KEYBDINPUT = std::mem::zeroed();
    ki.wVk = vk;
    ki.wScan = 0;
    ki.dwFlags = if is_key_down { 0 } else { KEYEVENTF_KEYUP };
    
    if is_extended {
        ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
    }
    
    ki.time = 0;
    ki.dwExtraInfo = INJECTED_KEY_MARKER; // マーカーを設定
    
    *input.u.ki_mut() = ki;
    
    let result = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
    
    if result == 0 {
        Err("Failed to send input".to_string())
    } else {
        Ok(())
    }
}

unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
    // ... (既存のコード)
    
    let raw_input = &*(buffer.as_ptr() as *const RAWINPUT);
    
    if raw_input.header.dwType == RIM_TYPEKEYBOARD {
        let keyboard = raw_input.data.keyboard();
        
        // 自分が送信したキーをスキップ
        if keyboard.ExtraInformation == INJECTED_KEY_MARKER {
            return None;
        }
        
        // ... (残りの処理)
    }
    
    None
}
```

### 成果物

- `send_key_event()` 関数
- `send_key()` 関数
- `is_extended_key()` 関数
- 無限ループ防止機能
- 更新された `process_raw_input` メソッド

### テスト方法

1. CapsLock → LCtrl のマッピングを設定
2. プログラムを起動
3. CapsLock キーを押す
4. LCtrl として動作することを確認（Ctrl+C でコピーなど）

### 見積もり時間

**5-6時間**

---

## Task 3.3: Swap モードの実装

### 目的

2つのキーを相互に入れ替える機能を実装します。例えば、CapsLock と LCtrl を入れ替えると、CapsLock を押すと LCtrl が送信され、LCtrl を押すと CapsLock が送信されます。

### 実装内容

#### 3.3.1: Swap マッピングの自動生成

Swap モードでは、双方向のマッピングを自動的に生成する必要があります。

**実装例**:
```rust
impl Config {
    fn add_mapping(&mut self, device_id: &str, from: String, to: String, mapping_type: MappingType) {
        self.add_device(device_id.to_string());
        if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == device_id) {
            // 既存のマッピングを削除
            device.mappings.retain(|m| m.from != from);
            
            if mapping_type == MappingType::Swap {
                // Swap モードの場合、逆マッピングも追加
                device.mappings.retain(|m| m.from != to);
                device.mappings.push(KeyMapping {
                    from: to.clone(),
                    to: from.clone(),
                    mapping_type: MappingType::Swap,
                });
            }
            
            device.mappings.push(KeyMapping {
                from,
                to,
                mapping_type,
            });
        }
    }
}
```

#### 3.3.2: 循環参照の検出

Swap マッピングで循環参照が発生しないようにチェックします。

**例**: A → B, B → C, C → A のような循環

**実装例**:
```rust
impl Config {
    fn check_circular_reference(&self, device_id: &str, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut current = to;
        
        while let Some(device) = self.devices.iter().find(|d| d.device_id == device_id) {
            if let Some(mapping) = device.mappings.iter().find(|m| m.from == current) {
                if mapping.to == from {
                    return true; // 循環参照を検出
                }
                
                if visited.contains(&mapping.to.as_str()) {
                    break; // 無限ループを防止
                }
                
                visited.insert(current);
                current = &mapping.to;
            } else {
                break;
            }
        }
        
        false
    }
    
    fn add_mapping(&mut self, device_id: &str, from: String, to: String, mapping_type: MappingType) {
        // 循環参照チェック
        if mapping_type == MappingType::Swap && self.check_circular_reference(device_id, &from, &to) {
            eprintln!("Warning: Circular reference detected, mapping not added");
            return;
        }
        
        // ... (既存のコード)
    }
}
```

#### 3.3.3: Swap モードのテスト

Swap モードが正しく動作することを確認するテストを追加します。

**実装例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_mapping() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Swap);
        
        let device = &config.devices[0];
        assert_eq!(device.mappings.len(), 2);
        
        // CapsLock -> LCtrl
        assert_eq!(device.mappings[0].from, "LCtrl");
        assert_eq!(device.mappings[0].to, "CapsLock");
        
        // LCtrl -> CapsLock
        assert_eq!(device.mappings[1].from, "CapsLock");
        assert_eq!(device.mappings[1].to, "LCtrl");
    }
    
    #[test]
    fn test_circular_reference_detection() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
        config.add_mapping("04FE:0021", "B".to_string(), "C".to_string(), MappingType::Swap);
        
        // A -> B, B -> C の状態で C -> A を追加しようとすると循環参照
        assert!(config.check_circular_reference("04FE:0021", "C", "A"));
    }
}
```

### 成果物

- 更新された `add_mapping` メソッド
- `check_circular_reference` メソッド
- Swap モードのテストケース

### テスト方法

1. CapsLock <-> LCtrl の Swap マッピングを設定
2. プログラムを起動
3. CapsLock キーを押すと LCtrl として動作することを確認
4. LCtrl キーを押すと CapsLock として動作することを確認

### 見積もり時間

**3-4時間**

---

## Task 3.4: Disable モードの実装

### 目的

特定のキーを無効化する機能を実装します。Disable モードでは、キー入力を抑制するだけで、リマップ先のキーは送信しません。

### 実装内容

#### 3.4.1: Disable モードの処理

`process_raw_input` で Disable モードを処理します。キーを抑制するだけで、送信は行いません。

**実装例**:
```rust
impl RawInputHandler {
    unsafe fn process_raw_input(&mut self, lparam: LPARAM) -> Option<String> {
        // ... (既存のコード)
        
        if let Some(mapping) = device.mappings.iter().find(|m| m.from == key_name) {
            match mapping.mapping_type {
                MappingType::Remap => {
                    send_key(&mapping.to, is_pressed).ok();
                    return Some(format!("{} -> {}", key_name, mapping.to));
                }
                MappingType::Swap => {
                    send_key(&mapping.to, is_pressed).ok();
                    return Some(format!("{} <-> {}", key_name, mapping.to));
                }
                MappingType::Disable => {
                    // キーを抑制するだけ（送信しない）
                    return Some(format!("{} (disabled)", key_name));
                }
            }
        }
        
        // ... (残りの処理)
    }
}
```

#### 3.4.2: Disable モードの設定検証

Disable モードでは、`to` フィールドは使用されませんが、設定ファイルの整合性のために空文字列または "None" を設定します。

**実装例**:
```rust
impl Config {
    fn add_mapping(&mut self, device_id: &str, from: String, to: String, mapping_type: MappingType) {
        // Disable モードの場合、to は "None" に設定
        let to = if mapping_type == MappingType::Disable {
            "None".to_string()
        } else {
            to
        };
        
        // ... (既存のコード)
    }
}
```

#### 3.4.3: Disable モードのテスト

Disable モードが正しく動作することを確認するテストを追加します。

**実装例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_mapping() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock".to_string(), "None".to_string(), MappingType::Disable);
        
        let device = &config.devices[0];
        assert_eq!(device.mappings.len(), 1);
        assert_eq!(device.mappings[0].from, "CapsLock");
        assert_eq!(device.mappings[0].to, "None");
        assert_eq!(device.mappings[0].mapping_type, MappingType::Disable);
    }
}
```

### 成果物

- 更新された `process_raw_input` メソッド
- 更新された `add_mapping` メソッド
- Disable モードのテストケース

### テスト方法

1. CapsLock を Disable に設定
2. プログラムを起動
3. CapsLock キーを押す
4. CapsLock が動作しないことを確認（LED が点灯しない、他のキーも送信されない）

### 見積もり時間

**2-3時間**

---

## Phase 3 統合テスト

### テストシナリオ

#### シナリオ1: Remap モードの動作確認

**設定**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
```

**テスト手順**:
1. プログラムを起動
2. CapsLock キーを押す
3. LCtrl として動作することを確認（Ctrl+C でコピーなど）

**期待される動作**:
- CapsLock キーを押すと LCtrl が送信される
- CapsLock LED は点灯しない
- LCtrl キーは通常通り動作する

#### シナリオ2: Swap モードの動作確認

**設定**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode swap
```

**テスト手順**:
1. プログラムを起動
2. CapsLock キーを押す → LCtrl として動作
3. LCtrl キーを押す → CapsLock として動作

**期待される動作**:
- CapsLock キーを押すと LCtrl が送信される
- LCtrl キーを押すと CapsLock が送信される（LED が点灯）
- 両方のキーが入れ替わる

#### シナリオ3: Disable モードの動作確認

**設定**:
```bash
cargo run -- set 04FE:0021 CapsLock None --mode disable
```

**テスト手順**:
1. プログラムを起動
2. CapsLock キーを押す
3. 何も起こらないことを確認

**期待される動作**:
- CapsLock キーを押しても何も起こらない
- CapsLock LED は点灯しない
- 他のキーは通常通り動作する

#### シナリオ4: 複数デバイスの動作確認

**設定**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- set 046D:C52B A B --mode remap
```

**テスト手順**:
1. 2つのキーボードを接続
2. プログラムを起動
3. 1つ目のキーボードで CapsLock を押す → LCtrl として動作
4. 2つ目のキーボードで A を押す → B として動作
5. 1つ目のキーボードで A を押す → A として動作（マッピングなし）

**期待される動作**:
- デバイスごとに異なるマッピングが適用される
- 他のデバイスには影響しない

#### シナリオ5: 複数キーの同時マッピング

**設定**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- set 04FE:0021 A B --mode remap
cargo run -- set 04FE:0021 Z Y --mode remap
```

**テスト手順**:
1. プログラムを起動
2. 各キーを押して、正しくマッピングされることを確認

**期待される動作**:
- CapsLock → LCtrl
- A → B
- Z → Y
- 他のキーは通常通り動作

### パフォーマンステスト

#### テスト1: 入力遅延の測定

**目標**: キー入力の遅延を 5ms 以下に抑える

**測定方法**:
1. 高速タイピングテストツールを使用
2. マッピングありとなしで比較
3. 遅延時間を測定

#### テスト2: CPU 使用率の測定

**目標**: バックグラウンドでの CPU 使用率を 1% 以下に抑える

**測定方法**:
1. プログラムを起動
2. タスクマネージャーで CPU 使用率を確認
3. キー入力時と待機時の使用率を測定

#### テスト3: 長時間稼働テスト

**目標**: 24時間以上の連続稼働でメモリリークやクラッシュが発生しないこと

**測定方法**:
1. プログラムを起動
2. 24時間放置
3. メモリ使用量とプロセス状態を確認

---

## Phase 3 完了基準

以下のすべての項目が完了した時点で、Phase 3 完了とします。

### 機能要件

- ✅ Low-level keyboard hook が正常に動作する
- ✅ キー入力が抑制される
- ✅ リマップ先のキーが送信される
- ✅ Remap モードが動作する
- ✅ Swap モードが動作する
- ✅ Disable モードが動作する
- ✅ 複数デバイスで異なるマッピングが適用される
- ✅ 無限ループが発生しない

### パフォーマンス要件

- ✅ キー入力の遅延が 5ms 以下
- ✅ CPU 使用率が 1% 以下
- ✅ メモリリークが発生しない

### テスト要件

- ✅ すべてのユニットテストが通過
- ✅ すべての統合テストが通過
- ✅ 実機でのテストが成功

### ドキュメント要件

- ✅ コード内にコメントが適切に記載されている
- ✅ Phase 3 完了レポートが作成されている

---

## v0.1.0-beta1 リリース準備

Phase 3 完了後、v0.1.0-beta1 をリリースします。

### リリース内容

**実装済み機能**:
- ✅ デバイス別キーマッピング設定
- ✅ リマップ・スワップ・無効化の3方式
- ✅ JSON設定ファイル対応
- ✅ CLI インターフェース
- ✅ Raw Input API 実装
- ✅ Low-level keyboard hook 実装
- ✅ 実際のキー入力のリマップ動作

**制限事項**:
- バックグラウンド実行は未実装（コンソールを閉じると終了）
- 設定のホットリロードは未実装
- GUI は未実装
- 修飾キー付きリマップは未実装（Ctrl+A など）

### リリース手順

1. Phase 3 完了レポートを作成
2. `feature/key-input-hook` ブランチを `main` にマージ
3. `v0.1.0-beta1` タグを作成
4. GitHub Actions でビルド
5. GitHub Releases でリリースを作成
6. リリースノートを記載

### リリースノート例

```markdown
# v0.1.0-beta1 - First Beta Release

Windows用デバイス別キーボードリマッパーの初回ベータリリースです。

## ✨ 新機能

- 実際のキーボード入力のリマップが動作します
- デバイス別のキーマッピング設定
- リマップ・スワップ・無効化の3方式
- 複数のキーボードを同時に使用可能

## 📦 ダウンロード

Windows バイナリは GitHub Actions からダウンロードしてください:
https://github.com/tkykszk/KeyboardRemapperR/actions/runs/XXXXXX

## 🚀 使用方法

1. ZIP ファイルを解凍
2. `keyboard-remapper-r.exe list` でキーボードを検出
3. `keyboard-remapper-r.exe set <device_id> <from> <to>` でマッピングを設定
4. `keyboard-remapper-r.exe start` でサービスを開始

## ⚠️ 制限事項

- コンソールを閉じるとサービスが終了します
- 設定変更にはサービスの再起動が必要です
- 管理者権限が必要です

## 🐛 既知の問題

- 一部のアプリケーションでキー入力が正しく認識されない場合があります
- 高速タイピング時に入力が遅延する場合があります

## 📝 フィードバック

バグ報告や機能要望は、GitHub の Issues ページで報告してください:
https://github.com/tkykszk/KeyboardRemapperR/issues
```

---

## 次のステップ（Phase 4）

Phase 3 完了後、Phase 4「サービス化と管理機能」に進みます。

### Phase 4 の主要タスク

**Task 4.1: バックグラウンド実行の実装**

コンソールを閉じても動作し続けるように、トレイアイコンまたは Windows サービスとして実装します。

**Task 4.2: Start/Stop コマンドの実装**

サービスの開始・停止を制御するコマンドを実装します。プロセス間通信を使用して、実行中のサービスと通信します。

**Task 4.3: 設定のホットリロード**

サービス再起動なしで設定を反映する機能を実装します。ファイル監視を使用して、設定ファイルの変更を検出します。

**Task 4.4: ログ機能の実装**

デバッグとトラブルシューティングを容易にするため、ログ機能を実装します。

---

## 必要な追加依存関係

```toml
[dependencies]
# 既存
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
once_cell = "1.19"

# Phase 3 で追加
# (特になし、既存の winapi で対応可能)

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winuser",
    "windef",
    "minwindef",
    "hidusage",
    "hidsdi",
    "libloaderapi",
] }
```

---

## 実装時の注意事項

### 1. 管理者権限

Low-level keyboard hook には管理者権限が必要です。プログラム起動時に権限チェックを行い、必要に応じてエラーメッセージを表示します。

### 2. セキュリティ

キーロガーと誤認される可能性があるため、ウイルス対策ソフトに検出される場合があります。デジタル署名を追加することを検討します。

### 3. パフォーマンス

キー入力イベントは高頻度で発生するため、処理を可能な限り高速化します。重い処理は別スレッドで実行します。

### 4. エラーハンドリング

Windows API のエラーは適切に処理し、ユーザーにわかりやすいエラーメッセージを表示します。

---

## まとめ

Phase 3 では、キー入力抑制と送信機能を実装し、実際のキーリマップ機能を実現します。これにより、KeyboardRemapperR の中核機能が完成し、v0.1.0-beta1 をリリースできます。

**見積もり時間**: 16-21時間  
**実装順序**: Task 3.1 → Task 3.2 → Task 3.4 → Task 3.3  
**次のマイルストーン**: Phase 3 完了 → v0.1.0-beta1 リリース

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: 1.0
