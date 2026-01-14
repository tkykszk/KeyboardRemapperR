# Phase 3 テスト計画

**作成日**: 2026年1月14日  
**対象**: Phase 3 - キー入力送信の実装  
**テスト戦略**: 単体テスト、統合テスト、モックテスト、パフォーマンステスト

---

## 📋 テスト概要

Phase 3 では、実際のキーリマップ機能を実現するため、**キー入力抑制**と**キー入力送信**の機能を実装します。これらの機能は Windows API に強く依存するため、モックを使用した単体テストと、実機での統合テストを組み合わせます。

---

## 🎯 テスト目標

### 主要目標

1. **機能の正確性**: すべてのマッピングモード（Remap/Swap/Disable）が正しく動作する
2. **パフォーマンス**: キー入力遅延が 5ms 以下
3. **安定性**: 長時間稼働（24時間）でメモリリークやクラッシュがない
4. **互換性**: 複数のキーボードデバイスで正しく動作する
5. **エッジケース**: 高頻度入力、同時押し、循環参照などの特殊ケースに対応

### Phase 3 完了基準

- ✅ すべての単体テストが通過（30+個）
- ✅ すべての統合テストが通過（10+個）
- ✅ すべてのパフォーマンステストが基準を満たす（5個）
- ✅ エッジケーステストが通過（10個）
- ✅ 実機テストで正常動作を確認

---

## 🧪 テストカテゴリ

### 1. 単体テスト（Unit Tests）

**目的**: 個別の関数やメソッドの動作を検証  
**テスト数**: 30個  
**実行環境**: Linux（モック使用）/ Windows（実API使用）

#### 1.1 キー入力抑制機能のテスト（8個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| UT-3.1.1 | `test_install_keyboard_hook` | Low-level keyboard hook のインストール | フックハンドルが返される |
| UT-3.1.2 | `test_uninstall_keyboard_hook` | Low-level keyboard hook のアンインストール | フックが正常に解除される |
| UT-3.1.3 | `test_add_suppressed_key` | 抑制キーリストへの追加 | キーがリストに追加される |
| UT-3.1.4 | `test_remove_suppressed_key` | 抑制キーリストからの削除 | キーがリストから削除される |
| UT-3.1.5 | `test_is_key_suppressed` | キーが抑制対象かチェック | 正しく判定される |
| UT-3.1.6 | `test_suppress_multiple_keys` | 複数キーの抑制 | すべてのキーが抑制される |
| UT-3.1.7 | `test_clear_suppressed_keys` | 抑制キーリストのクリア | リストが空になる |
| UT-3.1.8 | `test_hook_callback_suppression` | フックコールバックでの抑制 | 抑制対象キーが `1` を返す |

**実装方法**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_add_suppressed_key() {
        let mut handler = KeyboardHookHandler::new();
        handler.add_suppressed_key(VK_CAPITAL as i32);
        assert!(handler.is_key_suppressed(VK_CAPITAL as i32));
    }
}
```

#### 1.2 キー入力送信機能のテスト（8個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| UT-3.2.1 | `test_send_key_down` | キー押下イベントの送信 | SendInput が呼ばれる |
| UT-3.2.2 | `test_send_key_up` | キー解放イベントの送信 | SendInput が呼ばれる |
| UT-3.2.3 | `test_send_key_press` | キー押下→解放の送信 | 2回の SendInput が呼ばれる |
| UT-3.2.4 | `test_is_extended_key` | 拡張キーの判定 | 正しく判定される |
| UT-3.2.5 | `test_send_extended_key` | 拡張キーの送信 | `KEYEVENTF_EXTENDEDKEY` フラグが設定される |
| UT-3.2.6 | `test_send_key_by_name` | キー名からの送信 | VK コードに変換して送信される |
| UT-3.2.7 | `test_send_invalid_key_name` | 無効なキー名の処理 | エラーが返される |
| UT-3.2.8 | `test_prevent_infinite_loop` | 無限ループ防止マーカー | マーカーが設定される |

**実装方法**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_is_extended_key() {
        assert!(KeyInputSender::is_extended_key(VK_LEFT as i32));
        assert!(KeyInputSender::is_extended_key(VK_RIGHT as i32));
        assert!(!KeyInputSender::is_extended_key(VK_RETURN as i32));
    }
}
```

#### 1.3 Swap モードのテスト（6個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| UT-3.3.1 | `test_generate_swap_mapping` | Swap マッピングの自動生成 | 双方向マッピングが生成される |
| UT-3.3.2 | `test_detect_circular_reference_simple` | 単純な循環参照の検出 | A→B, B→A が検出される |
| UT-3.3.3 | `test_detect_circular_reference_complex` | 複雑な循環参照の検出 | A→B→C→A が検出される |
| UT-3.3.4 | `test_no_circular_reference` | 循環参照なしのケース | エラーが発生しない |
| UT-3.3.5 | `test_swap_mapping_overwrite` | Swap マッピングの上書き | 既存のマッピングが上書きされる |
| UT-3.3.6 | `test_swap_with_remap` | Swap と Remap の混在 | 正しく処理される |

**実装方法**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_swap_mapping() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "A", "B", MappingType::Swap);
        
        // A → B のマッピングが存在
        assert!(config.has_mapping("04FE:0021", "A", "B"));
        // B → A のマッピングも自動生成される
        assert!(config.has_mapping("04FE:0021", "B", "A"));
    }
}
```

#### 1.4 Disable モードのテスト（4個)

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| UT-3.4.1 | `test_add_disable_mapping` | Disable マッピングの追加 | マッピングが追加される |
| UT-3.4.2 | `test_process_disable_key` | Disable キーの処理 | キーが抑制される |
| UT-3.4.3 | `test_disable_multiple_keys` | 複数キーの無効化 | すべてのキーが無効化される |
| UT-3.4.4 | `test_disable_with_remap` | Disable と Remap の混在 | 正しく処理される |

**実装方法**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_process_disable_key() {
        let mut config = Config::new();
        config.add_mapping("04FE:0021", "CapsLock", "", MappingType::Disable);
        
        let result = config.process_key_event("04FE:0021", "CapsLock", true);
        assert_eq!(result, Some("None".to_string()));
    }
}
```

#### 1.5 統合機能のテスト（4個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| UT-3.5.1 | `test_remap_flow` | Remap の完全なフロー | 抑制→送信が正しく動作 |
| UT-3.5.2 | `test_swap_flow` | Swap の完全なフロー | 双方向マッピングが動作 |
| UT-3.5.3 | `test_disable_flow` | Disable の完全なフロー | キーが完全に無効化 |
| UT-3.5.4 | `test_no_mapping_flow` | マッピングなしのフロー | キーがそのまま通過 |

---

### 2. 統合テスト（Integration Tests）

**目的**: エンドツーエンドの動作を検証  
**テスト数**: 10個  
**実行環境**: Windows（実機）

#### 2.1 基本機能のテスト（4個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| IT-3.1.1 | `test_e2e_remap_capslock_to_lctrl` | CapsLock → LCtrl のリマップ | CapsLock を押すと LCtrl が送信される |
| IT-3.1.2 | `test_e2e_swap_a_and_b` | A ↔ B のスワップ | A を押すと B、B を押すと A が送信される |
| IT-3.1.3 | `test_e2e_disable_capslock` | CapsLock の無効化 | CapsLock を押しても何も起こらない |
| IT-3.1.4 | `test_e2e_no_mapping` | マッピングなし | キーがそのまま送信される |

**実装方法**:
```rust
#[test]
#[ignore] // 実機でのみ実行
fn test_e2e_remap_capslock_to_lctrl() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "CapsLock", "LCtrl", MappingType::Remap);
    
    // Start the service
    let handle = start_service(config);
    
    // Simulate CapsLock key press
    simulate_key_press(VK_CAPITAL);
    
    // Verify LCtrl was sent
    assert!(was_key_sent(VK_LCONTROL));
    
    // Stop the service
    stop_service(handle);
}
```

#### 2.2 複数デバイスのテスト（2個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| IT-3.2.1 | `test_multiple_devices_different_mappings` | 複数デバイスで異なるマッピング | デバイスごとに正しいマッピングが適用される |
| IT-3.2.2 | `test_device_hotplug` | デバイスの抜き差し | デバイスの抜き差し後も正常動作 |

#### 2.3 複雑なマッピングのテスト（4個）

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| IT-3.3.1 | `test_multiple_remaps` | 複数のリマップ | すべてのリマップが正しく動作 |
| IT-3.3.2 | `test_chain_remaps` | チェーンリマップ（A→B, B→C） | A を押すと C が送信される |
| IT-3.3.3 | `test_mixed_modes` | Remap/Swap/Disable の混在 | すべてのモードが正しく動作 |
| IT-3.3.4 | `test_config_reload` | 設定のリロード | リロード後も正常動作 |

---

### 3. パフォーマンステスト（Performance Tests）

**目的**: パフォーマンス要件を満たすことを検証  
**テスト数**: 5個  
**実行環境**: Windows（実機）

#### 3.1 パフォーマンステスト

| テストID | テスト名 | テスト内容 | 目標値 |
|---------|---------|----------|-------|
| PT-3.1 | `test_key_input_latency` | キー入力遅延の測定 | ≤5ms |
| PT-3.2 | `test_cpu_usage` | CPU 使用率の測定 | ≤1% |
| PT-3.3 | `test_memory_usage` | メモリ使用量の測定 | ≤10MB |
| PT-3.4 | `test_long_running_stability` | 長時間稼働テスト（24時間） | メモリリークなし |
| PT-3.5 | `test_high_frequency_input` | 高頻度入力テスト（100回/秒） | 取りこぼし ≤1% |

**実装方法**:
```rust
#[test]
#[ignore] // 実機でのみ実行
fn test_key_input_latency() {
    let mut config = Config::new();
    config.add_mapping("04FE:0021", "A", "B", MappingType::Remap);
    
    let handle = start_service(config);
    
    let mut latencies = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        simulate_key_press(VK_A);
        wait_for_key_sent(VK_B);
        let latency = start.elapsed();
        latencies.push(latency);
    }
    
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len();
    assert!(avg_latency < Duration::from_millis(5), "Average latency: {:?}", avg_latency);
    
    stop_service(handle);
}
```

---

### 4. エッジケーステスト（Edge Case Tests）

**目的**: 特殊なケースや境界条件を検証  
**テスト数**: 10個  
**実行環境**: Windows（実機）

#### 4.1 エッジケーステスト

| テストID | テスト名 | テスト内容 | 期待される結果 |
|---------|---------|----------|--------------|
| ET-3.1 | `test_simultaneous_key_press` | 同時押し（Ctrl+A） | 両方のキーが正しく処理される |
| ET-3.2 | `test_rapid_key_press` | 高速連打 | すべてのキーが処理される |
| ET-3.3 | `test_key_held_down` | キー長押し | リピートイベントが正しく処理される |
| ET-3.4 | `test_invalid_device_id` | 無効なデバイスID | エラーが適切に処理される |
| ET-3.5 | `test_empty_mapping` | 空のマッピング | エラーが適切に処理される |
| ET-3.6 | `test_self_mapping` | 自己マッピング（A→A） | 無限ループが発生しない |
| ET-3.7 | `test_circular_reference` | 循環参照（A→B→A） | エラーが検出される |
| ET-3.8 | `test_unmapped_device` | マッピングされていないデバイス | キーがそのまま通過 |
| ET-3.9 | `test_device_disconnect` | 実行中のデバイス切断 | クラッシュしない |
| ET-3.10 | `test_device_reconnect` | 実行中のデバイス再接続 | 自動的に認識される |

---

## 🛠️ テスト実装戦略

### モックの使用

Windows API を直接呼び出すテストは実機でのみ実行できるため、Linux 環境ではモックを使用します。

**モック対象の API**:
- `SetWindowsHookEx`: Low-level keyboard hook のインストール
- `UnhookWindowsHookEx`: Low-level keyboard hook のアンインストール
- `SendInput`: キー入力の送信
- `CallNextHookEx`: 次のフックへの伝播

**モック実装例**:
```rust
#[cfg(test)]
mod mock {
    pub struct MockKeyboardHook {
        suppressed_keys: Vec<i32>,
    }
    
    impl MockKeyboardHook {
        pub fn new() -> Self {
            MockKeyboardHook {
                suppressed_keys: Vec::new(),
            }
        }
        
        pub fn add_suppressed_key(&mut self, vk: i32) {
            self.suppressed_keys.push(vk);
        }
        
        pub fn is_key_suppressed(&self, vk: i32) -> bool {
            self.suppressed_keys.contains(&vk)
        }
    }
}
```

### テスト実行環境

**Linux 環境（開発時）**:
- 単体テスト（モック使用）
- コンパイルチェック
- 静的解析（clippy）

**Windows 環境（CI/実機）**:
- すべての単体テスト（実API使用）
- 統合テスト
- パフォーマンステスト
- エッジケーステスト

### テスト自動化

**GitHub Actions ワークフロー**:
```yaml
name: Phase 3 Tests

on: [push, pull_request]

jobs:
  test-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test --lib
      - name: Run integration tests
        run: cargo test --test e2e_tests
      - name: Run performance tests
        run: cargo test --test performance_tests -- --ignored
```

---

## 📊 テスト実行計画

### フェーズ1: 単体テスト（Week 1）

**目標**: すべての単体テストを実装し、通過させる

1. **Day 1-2**: キー入力抑制機能のテスト（8個）
2. **Day 3-4**: キー入力送信機能のテスト（8個）
3. **Day 5**: Swap モードのテスト（6個）
4. **Day 6**: Disable モードのテスト（4個）
5. **Day 7**: 統合機能のテスト（4個）

**期待される結果**: 30/30 テストが通過

### フェーズ2: 統合テスト（Week 2）

**目標**: エンドツーエンドの動作を確認

1. **Day 1-2**: 基本機能のテスト（4個）
2. **Day 3**: 複数デバイスのテスト（2個）
3. **Day 4-5**: 複雑なマッピングのテスト（4個）

**期待される結果**: 10/10 テストが通過

### フェーズ3: パフォーマンステスト（Week 2-3）

**目標**: パフォーマンス要件を満たすことを確認

1. **Day 6**: キー入力遅延、CPU、メモリのテスト（3個）
2. **Day 7**: 高頻度入力テスト（1個）
3. **Week 3**: 長時間稼働テスト（1個、24時間）

**期待される結果**: 5/5 テストが基準を満たす

### フェーズ4: エッジケーステスト（Week 3）

**目標**: 特殊なケースに対応できることを確認

1. **Day 1-2**: 同時押し、高速連打、長押しのテスト（3個）
2. **Day 3**: 無効な入力のテスト（3個）
3. **Day 4**: 循環参照のテスト（2個）
4. **Day 5**: デバイス抜き差しのテスト（2個）

**期待される結果**: 10/10 テストが通過

---

## 📈 テスト完了基準

### 必須項目

- ✅ すべての単体テストが通過（30/30）
- ✅ すべての統合テストが通過（10/10）
- ✅ すべてのパフォーマンステストが基準を満たす（5/5）
- ✅ 重大なエッジケースが通過（最低 8/10）

### パフォーマンス基準

- ✅ キー入力遅延: ≤5ms
- ✅ CPU 使用率: ≤1%
- ✅ メモリ使用量: ≤10MB
- ✅ 24時間稼働でメモリリークなし
- ✅ 高頻度入力（100回/秒）で取りこぼし ≤1%

### 品質基準

- ✅ コードカバレッジ: ≥80%
- ✅ すべての警告を解決
- ✅ clippy の警告なし
- ✅ ドキュメントの整備

---

## 🎯 v0.1.0-beta1 リリース基準

Phase 3 のテストがすべて通過したら、**v0.1.0-beta1** をリリースします。

**リリース基準**:
- ✅ Phase 1-3 の実装が完了
- ✅ すべてのテストが通過（55個）
- ✅ パフォーマンス基準を満たす
- ✅ ドキュメントが整備されている
- ✅ GitHub Actions が正常に動作

**実装済み機能**:
- ✅ デバイス別キーマッピング設定
- ✅ リマップ・スワップ・無効化の3方式
- ✅ 実際のキー入力のリマップ動作
- ✅ 複数デバイスの同時使用

**制限事項**:
- バックグラウンド実行は未実装（Phase 4）
- 設定のホットリロードは未実装（Phase 4）
- GUI は未実装（Phase 6）
- 修飾キー付きリマップは未実装（Phase 6）

---

## 📚 参考資料

### テストファイル

- `src/main.rs`: 単体テスト（`#[cfg(test)] mod tests`）
- `tests/phase3_unit_tests.rs`: Phase 3 の単体テスト
- `tests/e2e_tests.rs`: 統合テスト
- `tests/performance_tests.rs`: パフォーマンステスト

### テスト実行コマンド

```bash
# すべてのテストを実行
cargo test

# 単体テストのみ実行
cargo test --lib

# 統合テストのみ実行
cargo test --test e2e_tests

# パフォーマンステストを実行（実機のみ）
cargo test --test performance_tests -- --ignored

# カバレッジを測定
cargo tarpaulin --out Html
```

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: Phase 3
