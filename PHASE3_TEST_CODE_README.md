# Phase 3 テストコード

**作成日**: 2026年1月14日  
**対象**: Phase 3 - キー入力送信の実装  
**テストファイル**: `tests/phase3_unit_tests.rs`, `tests/phase3_integration_tests.rs`

---

## 📋 生成されたテストコード

Phase 3 のテスト計画（`PHASE3_TEST_PLAN.md`）を基に、以下のテストコードを生成しました。

### 1. 単体テスト（`tests/phase3_unit_tests.rs`）

**テスト数**: 30個  
**実行環境**: Windows（一部は Linux でモック使用）

#### テストカテゴリ

| カテゴリ | テスト数 | テストID範囲 |
|---------|---------|------------|
| キー入力抑制機能 | 5 | UT-3.1.1 ~ UT-3.1.5 |
| キー入力送信機能 | 5 | UT-3.2.1 ~ UT-3.2.5 |
| Swap モード | 5 | UT-3.3.1 ~ UT-3.3.5 |
| Disable モード | 2 | UT-3.4.1 ~ UT-3.4.2 |
| パフォーマンステスト | 3 | PT-3.1, PT-3.4, PT-3.5 |

#### 主要なテスト

**キー入力抑制機能**:
- `ut_3_1_1_install_keyboard_hook`: Low-level keyboard hook のインストール
- `ut_3_1_2_uninstall_keyboard_hook`: フックのアンインストール
- `ut_3_1_3_add_suppressed_key`: 抑制キーリストへの追加
- `ut_3_1_4_remove_suppressed_key`: 抑制キーリストからの削除
- `ut_3_1_5_multiple_suppressed_keys`: 複数キーの管理

**キー入力送信機能**:
- `ut_3_2_1_send_key_event`: キーイベントの送信
- `ut_3_2_2_is_extended_key`: 拡張キーの判定
- `ut_3_2_3_send_key`: キー名からの送信
- `ut_3_2_4_send_key_invalid`: 無効なキー名の処理
- `ut_3_2_5_injected_key_marker`: 無限ループ防止マーカー

**Swap モード**:
- `ut_3_3_1_swap_mapping_generation`: 双方向マッピングの自動生成
- `ut_3_3_2_circular_reference_simple`: 単純な循環参照の検出
- `ut_3_3_3_circular_reference_complex`: 複雑な循環参照の検出
- `ut_3_3_4_no_circular_reference`: 循環参照なしのケース
- `ut_3_3_5_swap_mapping_overwrite`: マッピングの上書き

**Disable モード**:
- `ut_3_4_1_disable_mapping`: Disable マッピングの追加
- `ut_3_4_2_disable_mapping_processing`: Disable キーの処理

**パフォーマンステスト**:
- `pt_3_1_key_input_latency`: キー入力遅延の測定（目標: ≤5ms）
- `pt_3_4_long_running_test`: 長時間稼働テスト（24時間、`#[ignore]`）
- `pt_3_5_high_frequency_input_test`: 高頻度入力テスト（100回/秒）

### 2. 統合テスト（`tests/phase3_integration_tests.rs`）

**テスト数**: 20個  
**実行環境**: Windows（実機）

#### テストカテゴリ

| カテゴリ | テスト数 | テストID範囲 |
|---------|---------|------------|
| 基本機能 | 4 | IT-3.1.1 ~ IT-3.1.4 |
| 複数デバイス | 2 | IT-3.2.1 ~ IT-3.2.2 |
| 複雑なマッピング | 4 | IT-3.3.1 ~ IT-3.3.4 |
| エッジケース | 10 | ET-3.1 ~ ET-3.10 |

#### 主要なテスト

**基本機能**:
- `it_3_1_1_e2e_remap_capslock_to_lctrl`: CapsLock → LCtrl のリマップ
- `it_3_1_2_e2e_swap_a_and_b`: A ↔ B のスワップ
- `it_3_1_3_e2e_disable_capslock`: CapsLock の無効化
- `it_3_1_4_e2e_no_mapping`: マッピングなし（パススルー）

**複数デバイス**:
- `it_3_2_1_multiple_devices_different_mappings`: 異なるマッピング
- `it_3_2_2_device_hotplug`: デバイスの抜き差し

**複雑なマッピング**:
- `it_3_3_1_multiple_remaps`: 複数のリマップ
- `it_3_3_2_chain_remaps`: チェーンリマップ（A→B, B→C）
- `it_3_3_3_mixed_modes`: Remap/Swap/Disable の混在
- `it_3_3_4_config_reload`: 設定のリロード

**エッジケース**:
- `et_3_1_simultaneous_key_press`: 同時押し（Ctrl+A）
- `et_3_2_rapid_key_press`: 高速連打
- `et_3_3_key_held_down`: キー長押し
- `et_3_4_invalid_device_id`: 無効なデバイスID
- `et_3_5_empty_mapping`: 空のマッピング
- `et_3_6_self_mapping`: 自己マッピング（A→A）
- `et_3_7_circular_reference`: 循環参照（A→B→A）
- `et_3_8_unmapped_device`: マッピングされていないデバイス
- `et_3_9_device_disconnect`: 実行中のデバイス切断
- `et_3_10_device_reconnect`: 実行中のデバイス再接続

---

## 🚀 テストの実行方法

### すべてのテストを実行

```bash
cargo test
```

### 単体テストのみ実行

```bash
cargo test --test phase3_unit_tests
```

### 統合テストのみ実行

```bash
cargo test --test phase3_integration_tests
```

### パフォーマンステストを実行

```bash
# 通常のパフォーマンステスト
cargo test --test phase3_unit_tests pt_3_1_key_input_latency
cargo test --test phase3_unit_tests pt_3_5_high_frequency_input_test

# 長時間稼働テスト（24時間）
cargo test --test phase3_unit_tests pt_3_4_long_running_test -- --ignored --nocapture
```

### 特定のテストを実行

```bash
# キー入力抑制機能のテスト
cargo test --test phase3_unit_tests ut_3_1

# Swap モードのテスト
cargo test --test phase3_unit_tests ut_3_3

# エッジケーステスト
cargo test --test phase3_integration_tests et_3
```

### 実機テスト（`#[ignore]` 付きテスト）を実行

```bash
# すべての実機テストを実行
cargo test -- --ignored

# 特定の実機テストを実行
cargo test --test phase3_integration_tests it_3_1_1_e2e_remap_capslock_to_lctrl -- --ignored
```

---

## 📊 テストの特徴

### 1. モックを使用した単体テスト

Windows API を直接呼び出すテストは実機でのみ実行できるため、Linux 環境ではモックを使用します。

**モック構造体**:
- `MockKeyboardHook`: キーボードフックのモック
- `MockKeyInputSender`: キー入力送信のモック
- `SwapConfig`: Swap/Disable マッピングのモック

**利点**:
- Linux 環境でもテストを実行できる
- CI/CD パイプラインで自動テストが可能
- テストの実行速度が速い

### 2. 実機でのテスト

統合テストとエッジケーステストは、実際のキーボード入力を必要とするため、`#[ignore]` 属性を付けています。

**実行方法**:
```bash
cargo test -- --ignored
```

**注意事項**:
- Windows 環境が必要
- 実際のキーボードが必要
- 手動での検証が必要な場合がある

### 3. パフォーマンステスト

パフォーマンステストは、実機で実行し、測定値を出力します。

**測定項目**:
- キー入力遅延: ≤5ms
- CPU 使用率: ≤1%（手動測定）
- メモリ使用量: ≤10MB（手動測定）
- 長時間稼働: 24時間（メモリリークなし）
- 高頻度入力: 100回/秒（取りこぼし ≤1%）

---

## 🎯 テスト完了基準

### Phase 3 完了基準

- ✅ すべての単体テストが通過（30/30）
- ✅ すべての統合テストが通過（10/10）
- ✅ すべてのパフォーマンステストが基準を満たす（3/3）
- ✅ 重大なエッジケースが通過（最低 8/10）

### v0.1.0-beta1 リリース基準

- ✅ Phase 1-3 の実装が完了
- ✅ すべてのテストが通過（55個）
- ✅ パフォーマンス基準を満たす
- ✅ ドキュメントが整備されている
- ✅ GitHub Actions が正常に動作

---

## 📝 テストコードの構造

### 単体テスト（`tests/phase3_unit_tests.rs`）

```rust
#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_unit_tests {
    // キー入力抑制機能のテスト
    #[test]
    fn ut_3_1_1_install_keyboard_hook() { ... }
    
    // キー入力送信機能のテスト
    #[test]
    fn ut_3_2_1_send_key_event() { ... }
    
    // Swap モードのテスト
    #[test]
    fn ut_3_3_1_swap_mapping_generation() { ... }
    
    // Disable モードのテスト
    #[test]
    fn ut_3_4_1_disable_mapping() { ... }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_performance_tests {
    // パフォーマンステスト
    #[test]
    fn pt_3_1_key_input_latency() { ... }
    
    #[test]
    #[ignore]
    fn pt_3_4_long_running_test() { ... }
}
```

### 統合テスト（`tests/phase3_integration_tests.rs`）

```rust
#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_integration_tests {
    // 基本機能のテスト
    #[test]
    #[ignore]
    fn it_3_1_1_e2e_remap_capslock_to_lctrl() { ... }
    
    // 複数デバイスのテスト
    #[test]
    #[ignore]
    fn it_3_2_1_multiple_devices_different_mappings() { ... }
    
    // 複雑なマッピングのテスト
    #[test]
    #[ignore]
    fn it_3_3_1_multiple_remaps() { ... }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod phase3_edge_case_tests {
    // エッジケーステスト
    #[test]
    #[ignore]
    fn et_3_1_simultaneous_key_press() { ... }
    
    #[test]
    fn et_3_4_invalid_device_id() { ... }
}
```

---

## 🔧 テストの実装状況

### 実装済み

- ✅ 単体テストの骨格（30個）
- ✅ 統合テストの骨格（10個）
- ✅ パフォーマンステストの骨格（3個）
- ✅ エッジケーステストの骨格（10個）

### 未実装（Phase 3 の実装後に完成）

- ⏳ 実際の Windows API 呼び出し
- ⏳ キーボード入力のシミュレーション
- ⏳ 送信されたキーの検証
- ⏳ サービスの起動/停止

---

## 📚 参考資料

### 関連ドキュメント

- `PHASE3_TASKS.md`: Phase 3 の実装タスク
- `PHASE3_TEST_CASES.md`: Phase 3 の詳細テストケース
- `PHASE3_TEST_PLAN.md`: Phase 3 のテスト計画
- `TEST_AUTOMATION_README.md`: テスト自動化の README

### テスト実行スクリプト

- `scripts/run_tests.ps1`: Windows 用テスト実行スクリプト
- `scripts/run_tests.sh`: Linux 用テスト実行スクリプト

### GitHub Actions

- `.github/workflows/test.yml`: CI/CD ワークフロー

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: Phase 3
