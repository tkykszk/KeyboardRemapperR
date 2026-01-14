# Phase 3: キー入力送信の実装 - 詳細テストケース

**作成日**: 2026年1月14日  
**対象バージョン**: v0.1.0-beta1  
**テスト環境**: Windows 10/11 (64-bit)

---

## 📋 テスト概要

Phase 3 の実装内容を検証するための包括的なテストケースです。単体テスト、統合テスト、パフォーマンステスト、エッジケーステストを含みます。

### テストカテゴリ

| カテゴリ | テスト数 | 見積もり時間 |
|---------|---------|------------|
| 単体テスト | 25 | 2-3時間 |
| 統合テスト | 15 | 3-4時間 |
| パフォーマンステスト | 5 | 2-3時間 |
| エッジケーステスト | 10 | 2-3時間 |
| **合計** | **55** | **9-13時間** |

---

## 🧪 単体テスト (Unit Tests)

### UT-3.1: キー入力抑制機能のテスト

#### UT-3.1.1: Low-level keyboard hook のインストール

**目的**: `install_keyboard_hook()` が正常に動作することを確認

**テスト手順**:
1. `install_keyboard_hook()` を呼び出す
2. 戻り値が `Ok(())` であることを確認
3. グローバル変数 `KEYBOARD_HOOK` が `Some` であることを確認

**期待される結果**:
- フックが正常にインストールされる
- エラーが発生しない

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_install_keyboard_hook() {
    unsafe {
        let result = install_keyboard_hook();
        assert!(result.is_ok());
        assert!(KEYBOARD_HOOK.is_some());
        
        // クリーンアップ
        uninstall_keyboard_hook();
    }
}
```

#### UT-3.1.2: Low-level keyboard hook のアンインストール

**目的**: `uninstall_keyboard_hook()` が正常に動作することを確認

**テスト手順**:
1. `install_keyboard_hook()` を呼び出す
2. `uninstall_keyboard_hook()` を呼び出す
3. グローバル変数 `KEYBOARD_HOOK` が `None` であることを確認

**期待される結果**:
- フックが正常にアンインストールされる
- エラーが発生しない

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_uninstall_keyboard_hook() {
    unsafe {
        install_keyboard_hook().ok();
        uninstall_keyboard_hook();
        assert!(KEYBOARD_HOOK.is_none());
    }
}
```

#### UT-3.1.3: キー抑制リストへの追加

**目的**: `add_suppressed_key()` が正常に動作することを確認

**テスト手順**:
1. `add_suppressed_key(VK_CAPITAL)` を呼び出す
2. `should_suppress_key(VK_CAPITAL, true)` が `true` を返すことを確認

**期待される結果**:
- キーが抑制リストに追加される
- 抑制判定が正しく動作する

**実装例**:
```rust
#[test]
fn test_add_suppressed_key() {
    add_suppressed_key(VK_CAPITAL);
    assert!(should_suppress_key(VK_CAPITAL, true));
    
    // クリーンアップ
    remove_suppressed_key(VK_CAPITAL);
}
```

#### UT-3.1.4: キー抑制リストからの削除

**目的**: `remove_suppressed_key()` が正常に動作することを確認

**テスト手順**:
1. `add_suppressed_key(VK_CAPITAL)` を呼び出す
2. `remove_suppressed_key(VK_CAPITAL)` を呼び出す
3. `should_suppress_key(VK_CAPITAL, true)` が `false` を返すことを確認

**期待される結果**:
- キーが抑制リストから削除される
- 抑制判定が正しく動作する

**実装例**:
```rust
#[test]
fn test_remove_suppressed_key() {
    add_suppressed_key(VK_CAPITAL);
    remove_suppressed_key(VK_CAPITAL);
    assert!(!should_suppress_key(VK_CAPITAL, true));
}
```

#### UT-3.1.5: 複数キーの抑制リスト管理

**目的**: 複数のキーを同時に管理できることを確認

**テスト手順**:
1. `add_suppressed_key(VK_CAPITAL)` を呼び出す
2. `add_suppressed_key(VK_A)` を呼び出す
3. 両方のキーが抑制されることを確認
4. 一方を削除して、もう一方は抑制されたままであることを確認

**期待される結果**:
- 複数のキーを独立して管理できる
- 一方の削除が他方に影響しない

**実装例**:
```rust
#[test]
fn test_multiple_suppressed_keys() {
    add_suppressed_key(VK_CAPITAL);
    add_suppressed_key(VK_A);
    
    assert!(should_suppress_key(VK_CAPITAL, true));
    assert!(should_suppress_key(VK_A, true));
    
    remove_suppressed_key(VK_CAPITAL);
    
    assert!(!should_suppress_key(VK_CAPITAL, true));
    assert!(should_suppress_key(VK_A, true));
    
    // クリーンアップ
    remove_suppressed_key(VK_A);
}
```

---

### UT-3.2: キー入力送信機能のテスト

#### UT-3.2.1: 基本的なキー送信

**目的**: `send_key_event()` が正常に動作することを確認

**テスト手順**:
1. `send_key_event(VK_A, true, false)` を呼び出す（キー押下）
2. 戻り値が `Ok(())` であることを確認
3. `send_key_event(VK_A, false, false)` を呼び出す（キー解放）
4. 戻り値が `Ok(())` であることを確認

**期待される結果**:
- キー送信が成功する
- エラーが発生しない

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_send_key_event() {
    unsafe {
        let result_down = send_key_event(VK_A, true, false);
        assert!(result_down.is_ok());
        
        let result_up = send_key_event(VK_A, false, false);
        assert!(result_up.is_ok());
    }
}
```

#### UT-3.2.2: 拡張キーの判定

**目的**: `is_extended_key()` が正しく判定することを確認

**テスト手順**:
1. 拡張キー（矢印キー、Home など）で `true` を返すことを確認
2. 通常キー（文字キーなど）で `false` を返すことを確認

**期待される結果**:
- 拡張キーが正しく判定される
- 通常キーが正しく判定される

**実装例**:
```rust
#[test]
fn test_is_extended_key() {
    // 拡張キー
    assert!(is_extended_key(0x25)); // Left Arrow
    assert!(is_extended_key(0x26)); // Up Arrow
    assert!(is_extended_key(0x27)); // Right Arrow
    assert!(is_extended_key(0x28)); // Down Arrow
    assert!(is_extended_key(0x24)); // Home
    assert!(is_extended_key(0x23)); // End
    assert!(is_extended_key(0xA3)); // Right Control
    assert!(is_extended_key(0xA5)); // Right Alt
    
    // 通常キー
    assert!(!is_extended_key(VK_A));
    assert!(!is_extended_key(VK_CAPITAL));
    assert!(!is_extended_key(VK_LCONTROL));
}
```

#### UT-3.2.3: キー名からの送信

**目的**: `send_key()` が正常に動作することを確認

**テスト手順**:
1. `send_key("A", true)` を呼び出す
2. 戻り値が `Ok(())` であることを確認
3. `send_key("CapsLock", true)` を呼び出す
4. 戻り値が `Ok(())` であることを確認

**期待される結果**:
- キー名から正しく送信される
- エラーが発生しない

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_send_key() {
    let result_a = send_key("A", true);
    assert!(result_a.is_ok());
    
    let result_caps = send_key("CapsLock", true);
    assert!(result_caps.is_ok());
}
```

#### UT-3.2.4: 無効なキー名のエラー処理

**目的**: 無効なキー名でエラーが返されることを確認

**テスト手順**:
1. `send_key("InvalidKey", true)` を呼び出す
2. 戻り値が `Err` であることを確認

**期待される結果**:
- エラーが返される
- エラーメッセージが適切

**実装例**:
```rust
#[test]
fn test_send_key_invalid() {
    let result = send_key("InvalidKey", true);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown key name"));
}
```

#### UT-3.2.5: 無限ループ防止マーカーの設定

**目的**: `dwExtraInfo` にマーカーが設定されることを確認

**テスト手順**:
1. `send_key_event()` 内で `dwExtraInfo` が `INJECTED_KEY_MARKER` に設定されることを確認

**期待される結果**:
- マーカーが正しく設定される

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_injected_key_marker() {
    // この テストは実装の詳細を確認するため、
    // 実際には process_raw_input でマーカーをチェックする統合テストで検証
    assert_eq!(INJECTED_KEY_MARKER, 0x12345678);
}
```

---

### UT-3.3: Swap モードのテスト

#### UT-3.3.1: Swap マッピングの自動生成

**目的**: Swap マッピングで双方向のマッピングが生成されることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "CapsLock", "LCtrl", MappingType::Swap)` を呼び出す
2. 2つのマッピングが生成されることを確認
3. CapsLock → LCtrl と LCtrl → CapsLock の両方が存在することを確認

**期待される結果**:
- 双方向のマッピングが自動生成される
- 両方のマッピングが正しい

**実装例**:
```rust
#[test]
fn test_swap_mapping_generation() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "CapsLock".to_string(), "LCtrl".to_string(), MappingType::Swap);
    
    let device = &config.devices[0];
    assert_eq!(device.mappings.len(), 2);
    
    // CapsLock -> LCtrl
    let mapping1 = device.mappings.iter().find(|m| m.from == "CapsLock");
    assert!(mapping1.is_some());
    assert_eq!(mapping1.unwrap().to, "LCtrl");
    
    // LCtrl -> CapsLock
    let mapping2 = device.mappings.iter().find(|m| m.from == "LCtrl");
    assert!(mapping2.is_some());
    assert_eq!(mapping2.unwrap().to, "CapsLock");
}
```

#### UT-3.3.2: 循環参照の検出 - 単純な循環

**目的**: A → B → A のような単純な循環が検出されることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "A", "B", MappingType::Swap)` を呼び出す
2. `config.check_circular_reference("04FE:0021", "B", "A")` が `true` を返すことを確認

**期待される結果**:
- 循環参照が検出される

**実装例**:
```rust
#[test]
fn test_circular_reference_simple() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
    
    // B -> A を追加しようとすると循環参照
    assert!(config.check_circular_reference("04FE:0021", "B", "A"));
}
```

#### UT-3.3.3: 循環参照の検出 - 複雑な循環

**目的**: A → B → C → A のような複雑な循環が検出されることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "A", "B", MappingType::Remap)` を呼び出す
2. `config.add_mapping("04FE:0021", "B", "C", MappingType::Remap)` を呼び出す
3. `config.check_circular_reference("04FE:0021", "C", "A")` が `true` を返すことを確認

**期待される結果**:
- 複雑な循環参照が検出される

**実装例**:
```rust
#[test]
fn test_circular_reference_complex() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Remap);
    config.add_mapping("04FE:0021", "B".to_string(), "C".to_string(), MappingType::Remap);
    
    // C -> A を追加しようとすると循環参照
    assert!(config.check_circular_reference("04FE:0021", "C", "A"));
}
```

#### UT-3.3.4: 循環参照なしのケース

**目的**: 循環参照がない場合に `false` が返されることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "A", "B", MappingType::Remap)` を呼び出す
2. `config.check_circular_reference("04FE:0021", "C", "D")` が `false` を返すことを確認

**期待される結果**:
- 循環参照が検出されない

**実装例**:
```rust
#[test]
fn test_no_circular_reference() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Remap);
    
    // C -> D は循環参照なし
    assert!(!config.check_circular_reference("04FE:0021", "C", "D"));
}
```

#### UT-3.3.5: Swap マッピングの上書き

**目的**: 既存の Swap マッピングが正しく上書きされることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "A", "B", MappingType::Swap)` を呼び出す
2. `config.add_mapping("04FE:0021", "A", "C", MappingType::Swap)` を呼び出す
3. A → C と C → A のマッピングのみが存在することを確認

**期待される結果**:
- 古いマッピングが削除される
- 新しいマッピングが追加される

**実装例**:
```rust
#[test]
fn test_swap_mapping_overwrite() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "A".to_string(), "B".to_string(), MappingType::Swap);
    config.add_mapping("04FE:0021", "A".to_string(), "C".to_string(), MappingType::Swap);
    
    let device = &config.devices[0];
    assert_eq!(device.mappings.len(), 2);
    
    // A -> C
    let mapping1 = device.mappings.iter().find(|m| m.from == "A");
    assert!(mapping1.is_some());
    assert_eq!(mapping1.unwrap().to, "C");
    
    // C -> A
    let mapping2 = device.mappings.iter().find(|m| m.from == "C");
    assert!(mapping2.is_some());
    assert_eq!(mapping2.unwrap().to, "A");
    
    // B -> A は存在しない
    let mapping3 = device.mappings.iter().find(|m| m.from == "B");
    assert!(mapping3.is_none());
}
```

---

### UT-3.4: Disable モードのテスト

#### UT-3.4.1: Disable マッピングの追加

**目的**: Disable マッピングが正しく追加されることを確認

**テスト手順**:
1. `config.add_mapping("04FE:0021", "CapsLock", "None", MappingType::Disable)` を呼び出す
2. マッピングが追加されることを確認
3. `to` フィールドが "None" であることを確認

**期待される結果**:
- Disable マッピングが追加される
- `to` フィールドが "None"

**実装例**:
```rust
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
```

#### UT-3.4.2: Disable マッピングの処理

**目的**: Disable マッピングでキーが送信されないことを確認

**テスト手順**:
1. Disable マッピングを設定
2. `process_key_event()` を呼び出す
3. 戻り値が適切であることを確認

**期待される結果**:
- キーが抑制される
- リマップ先のキーが送信されない

**実装例**:
```rust
#[test]
fn test_disable_mapping_processing() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "CapsLock".to_string(), "None".to_string(), MappingType::Disable);
    
    let result = config.process_key_event("04FE:0021", "CapsLock", true);
    assert!(result.is_some());
    assert!(result.unwrap().contains("disabled"));
}
```

---

## 🔗 統合テスト (Integration Tests)

### IT-3.1: Remap モードの統合テスト

#### IT-3.1.1: 基本的な Remap 動作

**目的**: CapsLock → LCtrl のリマップが正しく動作することを確認

**前提条件**:
- プログラムが起動している
- CapsLock → LCtrl のマッピングが設定されている

**テスト手順**:
1. CapsLock キーを押す
2. テキストエディタで Ctrl+C を実行
3. コピーが動作することを確認
4. CapsLock LED が点灯しないことを確認

**期待される結果**:
- CapsLock キーを押すと LCtrl として動作
- CapsLock LED は点灯しない
- LCtrl キーは通常通り動作

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- start
```

**検証方法**:
- テキストエディタで文字列を選択
- CapsLock + C でコピー
- CapsLock + V でペースト

#### IT-3.1.2: 文字キーの Remap

**目的**: A → B のリマップが正しく動作することを確認

**前提条件**:
- プログラムが起動している
- A → B のマッピングが設定されている

**テスト手順**:
1. テキストエディタを開く
2. A キーを押す
3. "B" が入力されることを確認

**期待される結果**:
- A キーを押すと "B" が入力される
- B キーは通常通り動作

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 A B --mode remap
cargo run -- start
```

#### IT-3.1.3: 複数キーの Remap

**目的**: 複数のキーが同時にリマップされることを確認

**前提条件**:
- プログラムが起動している
- CapsLock → LCtrl、A → B、Z → Y のマッピングが設定されている

**テスト手順**:
1. 各キーを押して、正しくリマップされることを確認

**期待される結果**:
- すべてのキーが正しくリマップされる
- 他のキーは通常通り動作

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- set 04FE:0021 A B --mode remap
cargo run -- set 04FE:0021 Z Y --mode remap
cargo run -- start
```

---

### IT-3.2: Swap モードの統合テスト

#### IT-3.2.1: 基本的な Swap 動作

**目的**: CapsLock ↔ LCtrl の Swap が正しく動作することを確認

**前提条件**:
- プログラムが起動している
- CapsLock ↔ LCtrl のマッピングが設定されている

**テスト手順**:
1. CapsLock キーを押す → LCtrl として動作することを確認
2. LCtrl キーを押す → CapsLock として動作することを確認（LED が点灯）

**期待される結果**:
- CapsLock キーを押すと LCtrl として動作
- LCtrl キーを押すと CapsLock として動作

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode swap
cargo run -- start
```

#### IT-3.2.2: 文字キーの Swap

**目的**: A ↔ B の Swap が正しく動作することを確認

**前提条件**:
- プログラムが起動している
- A ↔ B のマッピングが設定されている

**テスト手順**:
1. A キーを押す → "B" が入力される
2. B キーを押す → "A" が入力される

**期待される結果**:
- A キーを押すと "B" が入力される
- B キーを押すと "A" が入力される

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 A B --mode swap
cargo run -- start
```

---

### IT-3.3: Disable モードの統合テスト

#### IT-3.3.1: 基本的な Disable 動作

**目的**: CapsLock の無効化が正しく動作することを確認

**前提条件**:
- プログラムが起動している
- CapsLock の Disable マッピングが設定されている

**テスト手順**:
1. CapsLock キーを押す
2. 何も起こらないことを確認
3. CapsLock LED が点灯しないことを確認

**期待される結果**:
- CapsLock キーを押しても何も起こらない
- CapsLock LED は点灯しない

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock None --mode disable
cargo run -- start
```

#### IT-3.3.2: 複数キーの Disable

**目的**: 複数のキーが同時に無効化されることを確認

**前提条件**:
- プログラムが起動している
- CapsLock、A、Z の Disable マッピングが設定されている

**テスト手順**:
1. 各キーを押して、何も起こらないことを確認

**期待される結果**:
- すべてのキーが無効化される
- 他のキーは通常通り動作

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock None --mode disable
cargo run -- set 04FE:0021 A None --mode disable
cargo run -- set 04FE:0021 Z None --mode disable
cargo run -- start
```

---

### IT-3.4: 複数デバイスの統合テスト

#### IT-3.4.1: 2つのキーボードで異なるマッピング

**目的**: デバイスごとに異なるマッピングが適用されることを確認

**前提条件**:
- 2つのキーボードが接続されている
- プログラムが起動している
- デバイス1: CapsLock → LCtrl
- デバイス2: A → B

**テスト手順**:
1. デバイス1 で CapsLock を押す → LCtrl として動作
2. デバイス2 で A を押す → "B" が入力される
3. デバイス1 で A を押す → "A" が入力される（マッピングなし）
4. デバイス2 で CapsLock を押す → CapsLock として動作（マッピングなし）

**期待される結果**:
- デバイスごとに異なるマッピングが適用される
- 他のデバイスには影響しない

**テストコマンド**:
```bash
cargo run -- list
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- set 046D:C52B A B --mode remap
cargo run -- start
```

#### IT-3.4.2: 同じキーに異なるマッピング

**目的**: 同じキーでもデバイスごとに異なるマッピングが適用されることを確認

**前提条件**:
- 2つのキーボードが接続されている
- プログラムが起動している
- デバイス1: A → B
- デバイス2: A → C

**テスト手順**:
1. デバイス1 で A を押す → "B" が入力される
2. デバイス2 で A を押す → "C" が入力される

**期待される結果**:
- デバイスごとに異なるマッピングが適用される

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 A B --mode remap
cargo run -- set 046D:C52B A C --mode remap
cargo run -- start
```

---

### IT-3.5: 混合モードの統合テスト

#### IT-3.5.1: Remap + Swap + Disable の混合

**目的**: 異なるモードのマッピングが同時に動作することを確認

**前提条件**:
- プログラムが起動している
- CapsLock → LCtrl (Remap)
- A ↔ B (Swap)
- Z (Disable)

**テスト手順**:
1. CapsLock を押す → LCtrl として動作
2. A を押す → "B" が入力される
3. B を押す → "A" が入力される
4. Z を押す → 何も起こらない

**期待される結果**:
- すべてのモードが正しく動作する
- 相互に干渉しない

**テストコマンド**:
```bash
cargo run -- set 04FE:0021 CapsLock LCtrl --mode remap
cargo run -- set 04FE:0021 A B --mode swap
cargo run -- set 04FE:0021 Z None --mode disable
cargo run -- start
```

---

## ⚡ パフォーマンステスト (Performance Tests)

### PT-3.1: キー入力遅延の測定

**目的**: キー入力の遅延が 5ms 以下であることを確認

**テスト環境**:
- Windows 10/11 (64-bit)
- 標準的なUSBキーボード

**テスト手順**:
1. プログラムを起動
2. 高速タイピングテストツール（例: TypeRacer）を使用
3. マッピングありとなしで比較
4. 遅延時間を測定

**測定方法**:
- タイムスタンプを使用して、キー押下から送信までの時間を測定
- 100回の平均を取る

**期待される結果**:
- 平均遅延が 5ms 以下
- 最大遅延が 10ms 以下

**実装例**:
```rust
#[test]
#[cfg(target_os = "windows")]
fn test_key_input_latency() {
    use std::time::Instant;
    
    let mut total_duration = std::time::Duration::ZERO;
    let iterations = 100;
    
    for _ in 0..iterations {
        let start = Instant::now();
        
        unsafe {
            send_key_event(VK_A, true, false).ok();
            send_key_event(VK_A, false, false).ok();
        }
        
        let duration = start.elapsed();
        total_duration += duration;
    }
    
    let avg_duration = total_duration / iterations;
    println!("Average latency: {:?}", avg_duration);
    
    assert!(avg_duration.as_millis() <= 5);
}
```

---

### PT-3.2: CPU 使用率の測定

**目的**: バックグラウンドでの CPU 使用率が 1% 以下であることを確認

**テスト環境**:
- Windows 10/11 (64-bit)
- Intel Core i5 以上

**テスト手順**:
1. プログラムを起動
2. タスクマネージャーで CPU 使用率を確認
3. 待機時と入力時の使用率を測定

**測定方法**:
- 1分間の平均 CPU 使用率を測定
- キー入力なし（待機時）
- キー入力あり（通常使用時）

**期待される結果**:
- 待機時: 0.1% 以下
- 通常使用時: 1% 以下

---

### PT-3.3: メモリ使用量の測定

**目的**: メモリ使用量が 10MB 以下であることを確認

**テスト環境**:
- Windows 10/11 (64-bit)

**テスト手順**:
1. プログラムを起動
2. タスクマネージャーでメモリ使用量を確認
3. 起動直後と1時間後を比較

**測定方法**:
- プライベートワーキングセットを測定

**期待される結果**:
- 起動直後: 5MB 以下
- 1時間後: 10MB 以下（メモリリークなし）

---

### PT-3.4: 長時間稼働テスト

**目的**: 24時間以上の連続稼働でメモリリークやクラッシュが発生しないこと

**テスト環境**:
- Windows 10/11 (64-bit)

**テスト手順**:
1. プログラムを起動
2. 24時間放置
3. 定期的にキー入力を実行（自動化スクリプト）
4. メモリ使用量とプロセス状態を確認

**測定方法**:
- 1時間ごとにメモリ使用量を記録
- クラッシュやエラーログを確認

**期待される結果**:
- メモリ使用量が増加し続けない
- クラッシュが発生しない
- エラーログが記録されない

---

### PT-3.5: 高頻度入力テスト

**目的**: 高頻度のキー入力でも正常に動作することを確認

**テスト環境**:
- Windows 10/11 (64-bit)

**テスト手順**:
1. プログラムを起動
2. 自動化スクリプトで 1秒間に 100回のキー入力を実行
3. 10分間継続
4. 入力の取りこぼしがないことを確認

**測定方法**:
- 送信したキー数と受信したキー数を比較

**期待される結果**:
- 取りこぼしが 1% 以下
- クラッシュが発生しない

---

## 🔍 エッジケーステスト (Edge Case Tests)

### EC-3.1: 特殊キーのテスト

#### EC-3.1.1: ファンクションキーのリマップ

**目的**: F1-F12 キーが正しくリマップされることを確認

**テスト手順**:
1. F1 → F2 のマッピングを設定
2. F1 を押す
3. F2 として動作することを確認

**期待される結果**:
- ファンクションキーが正しくリマップされる

#### EC-3.1.2: 修飾キーのリマップ

**目的**: Shift、Ctrl、Alt キーが正しくリマップされることを確認

**テスト手順**:
1. LShift → RShift のマッピングを設定
2. LShift を押す
3. RShift として動作することを確認

**期待される結果**:
- 修飾キーが正しくリマップされる

#### EC-3.1.3: JIS キーボード固有キーのリマップ

**目的**: 変換、無変換キーが正しくリマップされることを確認

**テスト手順**:
1. 変換 → 無変換 のマッピングを設定
2. 変換を押す
3. 無変換として動作することを確認

**期待される結果**:
- JIS キーボード固有キーが正しくリマップされる

---

### EC-3.2: 同時押しのテスト

#### EC-3.2.1: 複数キーの同時押し

**目的**: 複数のキーを同時に押しても正常に動作することを確認

**テスト手順**:
1. A → B、C → D のマッピングを設定
2. A と C を同時に押す
3. B と D が同時に入力されることを確認

**期待される結果**:
- 複数のキーが正しくリマップされる
- 順序が保持される

#### EC-3.2.2: 修飾キー + 通常キーの同時押し

**目的**: Ctrl+A のような組み合わせが正常に動作することを確認

**テスト手順**:
1. A → B のマッピングを設定
2. Ctrl+A を押す
3. Ctrl+B として動作することを確認

**期待される結果**:
- 修飾キーの状態が保持される
- リマップが正しく適用される

---

### EC-3.3: 連打テスト

#### EC-3.3.1: 高速連打

**目的**: キーを高速で連打しても正常に動作することを確認

**テスト手順**:
1. A → B のマッピングを設定
2. A を高速で連打（1秒間に10回）
3. すべての入力が "B" として処理されることを確認

**期待される結果**:
- すべての入力が正しくリマップされる
- 取りこぼしがない

#### EC-3.3.2: 長押し

**目的**: キーを長押ししても正常に動作することを確認

**テスト手順**:
1. A → B のマッピングを設定
2. A を5秒間長押し
3. "BBBBB..." と連続入力されることを確認

**期待される結果**:
- リピート入力が正しくリマップされる

---

### EC-3.4: 設定変更のテスト

#### EC-3.4.1: 実行中の設定変更

**目的**: プログラム実行中に設定を変更しても正常に動作することを確認

**テスト手順**:
1. A → B のマッピングで起動
2. 設定ファイルを A → C に変更
3. プログラムを再起動
4. A を押すと "C" が入力されることを確認

**期待される結果**:
- 新しい設定が正しく適用される

#### EC-3.4.2: 無効な設定の処理

**目的**: 無効な設定ファイルでもクラッシュしないことを確認

**テスト手順**:
1. 設定ファイルを破損させる（JSON構文エラー）
2. プログラムを起動
3. エラーメッセージが表示されることを確認
4. プログラムがクラッシュしないことを確認

**期待される結果**:
- エラーメッセージが表示される
- プログラムがクラッシュしない
- デフォルト設定で起動する

---

### EC-3.5: デバイス接続/切断のテスト

#### EC-3.5.1: 実行中のデバイス切断

**目的**: プログラム実行中にキーボードを切断してもクラッシュしないことを確認

**テスト手順**:
1. プログラムを起動
2. キーボードを物理的に切断
3. プログラムがクラッシュしないことを確認

**期待される結果**:
- プログラムがクラッシュしない
- エラーログが記録される

#### EC-3.5.2: 実行中のデバイス接続

**目的**: プログラム実行中にキーボードを接続しても正常に動作することを確認

**テスト手順**:
1. プログラムを起動
2. 新しいキーボードを接続
3. `list` コマンドで新しいデバイスが検出されることを確認

**期待される結果**:
- 新しいデバイスが検出される
- 既存のマッピングは影響を受けない

---

## 📊 テスト結果の記録

### テスト結果テンプレート

```markdown
## テスト結果

**テスト日**: YYYY-MM-DD  
**テスター**: [名前]  
**環境**: Windows [バージョン]  
**ビルド**: [コミットハッシュ]

### 単体テスト

| テストID | テスト名 | 結果 | 備考 |
|---------|---------|------|------|
| UT-3.1.1 | Low-level keyboard hook のインストール | ✅ Pass | - |
| UT-3.1.2 | Low-level keyboard hook のアンインストール | ✅ Pass | - |
| ... | ... | ... | ... |

### 統合テスト

| テストID | テスト名 | 結果 | 備考 |
|---------|---------|------|------|
| IT-3.1.1 | 基本的な Remap 動作 | ✅ Pass | - |
| IT-3.1.2 | 文字キーの Remap | ✅ Pass | - |
| ... | ... | ... | ... |

### パフォーマンステスト

| テストID | テスト名 | 測定値 | 基準値 | 結果 |
|---------|---------|--------|--------|------|
| PT-3.1 | キー入力遅延 | 3.2ms | ≤5ms | ✅ Pass |
| PT-3.2 | CPU 使用率 | 0.8% | ≤1% | ✅ Pass |
| ... | ... | ... | ... | ... |

### エッジケーステスト

| テストID | テスト名 | 結果 | 備考 |
|---------|---------|------|------|
| EC-3.1.1 | ファンクションキーのリマップ | ✅ Pass | - |
| EC-3.1.2 | 修飾キーのリマップ | ✅ Pass | - |
| ... | ... | ... | ... |

### 総合評価

- **合格率**: XX/55 (XX%)
- **重大な問題**: なし
- **軽微な問題**: [問題の説明]
- **リリース判定**: ✅ リリース可能 / ❌ 修正が必要
```

---

## 🎯 テスト完了基準

以下のすべての項目が満たされた時点で、Phase 3 のテストが完了したとみなします。

### 必須項目

- ✅ すべての単体テストが通過（25/25）
- ✅ すべての統合テストが通過（15/15）
- ✅ すべてのパフォーマンステストが基準を満たす（5/5）
- ✅ 重大なエッジケースが通過（最低 8/10）

### パフォーマンス基準

- ✅ キー入力遅延: ≤5ms
- ✅ CPU 使用率: ≤1%
- ✅ メモリ使用量: ≤10MB
- ✅ 24時間稼働でメモリリークなし

### ドキュメント

- ✅ テスト結果が記録されている
- ✅ 発見された問題が Issue として登録されている
- ✅ テスト完了レポートが作成されている

---

## 🚀 次のステップ

テスト完了後、以下のアクションを実行します:

1. **テスト結果レポートの作成**: すべてのテスト結果をまとめたレポートを作成
2. **Issue の登録**: 発見された問題を GitHub Issues に登録
3. **修正の実施**: 重大な問題を修正
4. **再テスト**: 修正後に再度テストを実行
5. **v0.1.0-beta1 リリース**: すべてのテストが通過したらリリース

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: 1.0
