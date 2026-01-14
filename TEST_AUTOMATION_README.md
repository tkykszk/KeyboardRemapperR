# Phase 3 テスト自動化ガイド

**作成日**: 2026年1月14日  
**対象バージョン**: v0.1.0-beta1  
**対応OS**: Windows 10/11, Linux

---

## 📋 概要

Phase 3 の実装を検証するための包括的なテスト自動化システムです。単体テスト、統合テスト、パフォーマンステストを自動実行し、テスト結果をレポートとして出力します。

### テスト自動化の構成

| ファイル | 説明 |
|---------|------|
| `tests/phase3_unit_tests.rs` | Rust の単体テスト・パフォーマンステスト |
| `scripts/run_tests.ps1` | Windows 用テスト実行スクリプト |
| `scripts/run_tests.sh` | Linux 用テスト実行スクリプト |
| `PHASE3_TEST_CASES.md` | 詳細なテストケース仕様 |

---

## 🚀 クイックスタート

### Windows の場合

```powershell
# すべてのテストを実行
.\scripts\run_tests.ps1

# 単体テストのみ実行
.\scripts\run_tests.ps1 -TestType unit

# 統合テストのみ実行
.\scripts\run_tests.ps1 -TestType integration

# パフォーマンステストのみ実行
.\scripts\run_tests.ps1 -TestType performance

# レポートを生成
.\scripts\run_tests.ps1 -GenerateReport

# 詳細出力
.\scripts\run_tests.ps1 -Verbose
```

### Linux の場合

```bash
# すべてのテストを実行
./scripts/run_tests.sh

# 単体テストのみ実行
./scripts/run_tests.sh unit

# 統合テストのみ実行
./scripts/run_tests.sh integration

# パフォーマンステストのみ実行
./scripts/run_tests.sh performance

# レポートを生成
./scripts/run_tests.sh all --report

# 詳細出力
./scripts/run_tests.sh all --verbose
```

---

## 🧪 テストカテゴリ

### 単体テスト (Unit Tests)

**テスト数**: 25個  
**実行時間**: 2-3秒  
**コマンド**: `cargo test --lib`

**テスト内容**:
- UT-3.1: キー入力抑制機能（5個）
- UT-3.2: キー入力送信機能（5個）
- UT-3.3: Swap モード（5個）
- UT-3.4: Disable モード（2個）

**実装ファイル**: `tests/phase3_unit_tests.rs`

### 統合テスト (Integration Tests)

**テスト数**: 15個  
**実行時間**: 3-4秒  
**コマンド**: `cargo test --test e2e_tests`

**テスト内容**:
- IT-3.1: Remap モードの統合テスト（3個）
- IT-3.2: Swap モードの統合テスト（2個）
- IT-3.3: Disable モードの統合テスト（2個）
- IT-3.4: 複数デバイスの統合テスト（2個）
- IT-3.5: 混合モードの統合テスト（1個）

**実装ファイル**: `tests/e2e_tests.rs`（Phase 2 完了後に実装）

### パフォーマンステスト (Performance Tests)

**テスト数**: 5個  
**実行時間**: 2-3秒  
**コマンド**: `cargo test --release -- --ignored --nocapture`

**テスト内容**:
- PT-3.1: キー入力遅延の測定（目標: ≤5ms）
- PT-3.2: CPU 使用率の測定（目標: ≤1%）
- PT-3.3: メモリ使用量の測定（目標: ≤10MB）
- PT-3.4: 長時間稼働テスト（目標: 24時間以上）
- PT-3.5: 高頻度入力テスト（目標: 取りこぼし ≤1%）

**実装ファイル**: `tests/phase3_unit_tests.rs`

---

## 📊 テストレポート

テスト実行後、以下の形式でレポートが自動生成されます。

### レポートの構成

```markdown
# Phase 3 テスト結果レポート

**テスト日**: 2026-01-14 15:30:00  
**環境**: Windows 11  
**総実行時間**: 8.5秒

---

## 📊 テスト結果サマリー

| カテゴリ | 合計 | 通過 | 失敗 | 実行時間 | 合格率 |
|---------|------|------|------|---------|--------|
| 単体テスト | 25 | 25 | 0 | 2.3秒 | 100% |
| 統合テスト | 15 | 15 | 0 | 3.8秒 | 100% |
| パフォーマンステスト | 5 | 5 | 0 | 2.4秒 | 100% |

---

## ⚡ パフォーマンスメトリクス

| メトリクス | 測定値 | 基準値 | 結果 |
|-----------|--------|--------|------|
| キー入力遅延 | 3.2ms | ≤5ms | ✅ Pass |
| 成功率 | 99.8% | ≥99% | ✅ Pass |

---

## 🎯 総合評価

- **合格率**: 45/45 (100%)
- **重大な問題**: なし
- **リリース判定**: ✅ リリース可能
```

### レポートファイル名

`test_results_YYYYMMDD_HHMMSS.md`

例: `test_results_20260114_153000.md`

---

## 🔧 テストの実装方法

### 単体テストの追加

`tests/phase3_unit_tests.rs` に新しいテストを追加します。

```rust
#[test]
fn test_new_feature() {
    // テストコード
    assert!(true);
}
```

### 統合テストの追加

`tests/e2e_tests.rs` に新しいテストを追加します。

```rust
#[test]
fn test_new_integration() {
    // テストコード
    assert!(true);
}
```

### パフォーマンステストの追加

`tests/phase3_unit_tests.rs` の `phase3_performance_tests` モジュールに追加します。

```rust
#[test]
fn test_new_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    // パフォーマンステストコード
    let duration = start.elapsed();
    
    assert!(duration.as_millis() <= 10);
}
```

---

## 📈 テスト完了基準

Phase 3 のテストが完了したとみなす基準は以下の通りです。

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

## 🐛 トラブルシューティング

### テストが失敗する場合

1. **コンパイルエラー**: `cargo build` でビルドエラーを確認
2. **依存関係の問題**: `cargo update` で依存関係を更新
3. **環境の問題**: 管理者権限で実行（Low-level keyboard hook に必要）

### パフォーマンステストが基準を満たさない場合

1. **CPU 使用率が高い**: バックグラウンドプロセスを確認
2. **メモリ使用量が多い**: メモリリークを確認
3. **遅延が大きい**: デバッグビルドではなくリリースビルドを使用

### スクリプトが実行できない場合

**Windows の場合**:
```powershell
# 実行ポリシーを変更
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

**Linux の場合**:
```bash
# 実行権限を付与
chmod +x scripts/run_tests.sh
```

---

## 🔄 CI/CD 統合

### GitHub Actions

`.github/workflows/test.yml` に以下を追加:

```yaml
name: Phase 3 Tests

on:
  push:
    branches: [ main, feature/* ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: windows-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Run tests
      run: .\scripts\run_tests.ps1 -GenerateReport
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: test_results_*.md
```

---

## 📚 関連ドキュメント

- [PHASE3_TEST_CASES.md](PHASE3_TEST_CASES.md) - 詳細なテストケース仕様
- [PHASE3_TASKS.md](PHASE3_TASKS.md) - Phase 3 の実装タスク
- [IMPLEMENTATION_TASKS.md](IMPLEMENTATION_TASKS.md) - 全体の実装タスク

---

## 🎯 次のステップ

1. **Phase 2 の実装**: Phase 2 のタスクを完了
2. **統合テストの実装**: `tests/e2e_tests.rs` を作成
3. **CI/CD の設定**: GitHub Actions を設定
4. **v0.1.0-beta1 リリース**: すべてのテストが通過したらリリース

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: 1.0
