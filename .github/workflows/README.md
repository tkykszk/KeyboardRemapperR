# GitHub Actions ワークフロー

このディレクトリには、KeyboardRemapperR の CI/CD パイプラインを定義する GitHub Actions ワークフローが含まれています。

## 📋 ワークフロー一覧

### test.yml - Phase 3 テスト

**トリガー**:
- `push`: main, feature/*, develop ブランチへのプッシュ
- `pull_request`: main, develop ブランチへのプルリクエスト
- `workflow_dispatch`: 手動実行

**ジョブ**:

#### 1. test-windows
Windows 環境でテストを実行します。

**ステップ**:
1. **Checkout code**: リポジトリをチェックアウト
2. **Install Rust toolchain**: Rust ツールチェーンをインストール（stable）
3. **Cache cargo registry/index/target**: ビルド時間を短縮するためにキャッシュを使用
4. **Check code formatting**: `cargo fmt` でコードフォーマットをチェック
5. **Run clippy**: `cargo clippy` で静的解析を実行
6. **Build project**: プロジェクトをビルド
7. **Run unit tests**: 単体テストを実行
8. **Run integration tests**: 統合テストを実行
9. **Run all tests with script**: テスト自動化スクリプトを実行してレポートを生成
10. **Upload test results**: テスト結果を成果物としてアップロード
11. **Upload build artifacts**: ビルド成果物をアップロード

**成果物**:
- `test-results-windows`: テスト結果レポート（Markdown）
- `keyboard-remapper-r-windows`: デバッグビルドのバイナリ

#### 2. build-release
リリースビルドを作成します（test-windows が成功した場合のみ）。

**条件**:
- `test-windows` ジョブが成功
- `push` イベント
- `main` または `feature/*` ブランチ

**ステップ**:
1. **Checkout code**: リポジトリをチェックアウト
2. **Install Rust toolchain**: Rust ツールチェーンをインストール
3. **Cache cargo registry/index/target**: キャッシュを使用
4. **Build release binary**: リリースビルドを作成
5. **Get binary size**: バイナリサイズを取得
6. **Upload release binary**: リリースバイナリをアップロード
7. **Create release summary**: ビルドサマリーを作成

**成果物**:
- `keyboard-remapper-r-windows-release`: リリースビルドのバイナリ

---

## 🚀 使用方法

### 自動実行

ワークフローは以下の場合に自動実行されます:

1. **プッシュ時**: main, feature/*, develop ブランチにプッシュ
2. **プルリクエスト時**: main, develop ブランチへのプルリクエスト

### 手動実行

GitHub の Actions タブから手動で実行できます:

1. GitHub リポジトリの **Actions** タブを開く
2. 左側のワークフロー一覧から **Phase 3 Tests** を選択
3. **Run workflow** ボタンをクリック
4. ブランチを選択して **Run workflow** をクリック

---

## 📊 テスト結果の確認

### ワークフロー実行結果

1. GitHub リポジトリの **Actions** タブを開く
2. 実行したいワークフローをクリック
3. 各ジョブの詳細を確認

### テスト結果レポート

1. ワークフロー実行ページの下部にある **Artifacts** セクションを確認
2. `test-results-windows` をダウンロード
3. Markdown ファイルを開いてテスト結果を確認

### ビルド成果物

1. ワークフロー実行ページの下部にある **Artifacts** セクションを確認
2. `keyboard-remapper-r-windows` または `keyboard-remapper-r-windows-release` をダウンロード
3. バイナリを実行

---

## 🔧 ワークフローのカスタマイズ

### トリガーの変更

`test.yml` の `on` セクションを編集:

```yaml
on:
  push:
    branches: [ main, feature/*, develop, your-branch ]
  pull_request:
    branches: [ main, develop ]
  schedule:
    - cron: '0 0 * * *'  # 毎日午前0時に実行
```

### テストの追加

`test.yml` の `steps` セクションに新しいステップを追加:

```yaml
- name: Run custom tests
  run: cargo test --test my_custom_test --verbose
```

### 成果物の保持期間の変更

`retention-days` を変更:

```yaml
- name: Upload test results
  uses: actions/upload-artifact@v4
  with:
    name: test-results-windows
    path: test_results_*.md
    retention-days: 90  # 90日間保持
```

---

## 🎯 ステータスバッジ

README.md にステータスバッジを追加できます:

```markdown
[![Phase 3 Tests](https://github.com/tkykszk/KeyboardRemapperR/actions/workflows/test.yml/badge.svg)](https://github.com/tkykszk/KeyboardRemapperR/actions/workflows/test.yml)
```

---

## 🐛 トラブルシューティング

### ワークフローが失敗する場合

1. **ログを確認**: ワークフロー実行ページで各ステップのログを確認
2. **ローカルで再現**: 同じコマンドをローカル環境で実行
3. **キャッシュをクリア**: Actions タブの Settings から Cache を削除

### テストが失敗する場合

1. **テスト結果レポートをダウンロード**: Artifacts から `test-results-windows` をダウンロード
2. **詳細を確認**: レポートで失敗したテストの詳細を確認
3. **ローカルでテスト**: `.\scripts\run_tests.ps1 -Verbose` で詳細を確認

### ビルドが失敗する場合

1. **Rust のバージョンを確認**: `cargo --version` でバージョンを確認
2. **依存関係を更新**: `cargo update` で依存関係を更新
3. **クリーンビルド**: `cargo clean && cargo build` でクリーンビルド

---

## 📚 関連リンク

- [GitHub Actions ドキュメント](https://docs.github.com/en/actions)
- [actions/checkout](https://github.com/actions/checkout)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain)
- [actions/cache](https://github.com/actions/cache)
- [actions/upload-artifact](https://github.com/actions/upload-artifact)

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: 1.0
