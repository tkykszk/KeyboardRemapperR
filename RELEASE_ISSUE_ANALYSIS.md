# リリース作成の問題分析

## 現状

GitHub Actions でリリースワークフローが失敗し、v0.0.1-alpha1 のリリースが作成されていません。

### 失敗したワークフロー

| Run # | ワークフロー | ステータス | 結論 | URL |
|---|---|---|---|---|
| #4 | Release | completed | failure | https://github.com/tkykszk/KeyboardRemapperR/actions/runs/20973726864 |
| #3 | Release | completed | failure | https://github.com/tkykszk/KeyboardRemapperR/actions/runs/20973605206 |
| #2 | Release | completed | failure | https://github.com/tkykszk/KeyboardRemapperR/actions/runs/20973605206 |

### 問題

1. **YAML 構文エラー**: 最初の失敗は YAML 構文エラー (78行目)
2. **修正後も失敗**: YAML 構文エラーを修正したが、依然として失敗
3. **詳細ログなし**: GitHub Actions のログが非公開のため、詳細なエラーメッセージが確認できない

---

## 解決策

### 手動でリリースを作成

GitHub Actions が失敗しているため、手動でリリースを作成します。

#### ステップ 1: Windows バイナリを取得

GitHub Actions の成功したビルドから Windows バイナリをダウンロード:
- https://github.com/tkykszk/KeyboardRemapperR/actions/runs/20973219066

#### ステップ 2: リリースパッケージを作成

```powershell
# パッケージディレクトリを作成
$packageName = "KeyboardRemapperR-v0.0.1-alpha1-windows-x64"
New-Item -ItemType Directory -Path $packageName -Force

# バイナリをコピー
Copy-Item "keyboard-remapper-r.exe" "$packageName\"

# テストスクリプトをコピー
Copy-Item "tests\e2e_test_simple.ps1" "$packageName\"
Copy-Item "tests\e2e_test_full.ps1" "$packageName\"

# README をコピー
Copy-Item "README.md" "$packageName\"

# サンプル設定ファイルを作成
@"
{
  "devices": {
    "04FE:0021": {
      "name": "HHKB Professional",
      "mappings": [
        {
          "from": "CapsLock",
          "to": "LCtrl"
        }
      ]
    }
  }
}
"@ | Out-File -FilePath "$packageName\config.sample.json" -Encoding utf8

# ZIP を作成
Compress-Archive -Path $packageName -DestinationPath "$packageName.zip"
```

#### ステップ 3: GitHub で手動リリースを作成

1. GitHub リポジトリの Releases ページにアクセス:  
   https://github.com/tkykszk/KeyboardRemapperR/releases

2. "Draft a new release" をクリック

3. 以下の情報を入力:
   - **Tag**: `v0.0.1-alpha1`
   - **Release title**: `v0.0.1-alpha1 - Initial Alpha Release`
   - **Description**:
     ```markdown
     ## KeyboardRemapperR v0.0.1-alpha1 - Initial Alpha Release
     
     Windows用デバイス別キーボードリマッパーの初回アルファリリースです。
     
     ### ✨ 機能
     
     - ✅ デバイス別キーマッピング設定
     - ✅ リマップ・スワップ・無効化の3方式
     - ✅ JSON設定ファイル対応
     - ✅ CLI インターフェース
     - ✅ Raw Input API 実装
     - ✅ E2E テスト (6個の基本テスト)
     
     ### 📦 ダウンロード
     
     `KeyboardRemapperR-v0.0.1-alpha1-windows-x64.zip` をダウンロードして使用してください。
     
     ### 🚀 使用方法
     
     1. ZIP ファイルを解凍
     2. `keyboard-remapper-r.exe` を実行
     3. `list` コマンドでキーボードを検出
     4. `set` コマンドでマッピングを設定
     
     ### 📚 ドキュメント
     
     - **README.md**: 詳細な使い方
     - **E2E テスト**: `e2e_test_simple.ps1`, `e2e_test_full.ps1`
     
     ### ⚠️ 注意事項
     
     これはアルファ版です。以下の制限があります:
     
     - Raw Input API の実装は完全ではありません
     - 実際のキーボード入力フックは未実装
     - デバイス ID の抽出は未実装
     
     ### 🐛 既知の問題
     
     - デバイス検出機能はシミュレーションのみ
     - キーマッピングは設定ファイルに保存されますが、実際のキー入力には適用されません
     
     ### 📝 フィードバック
     
     バグ報告や機能要望は、GitHub の Issues ページで報告してください:  
     https://github.com/tkykszk/KeyboardRemapperR/issues
     ```

4. "This is a pre-release" をチェック

5. ZIP ファイルをアップロード

6. "Publish release" をクリック

---

## 今後の対応

### リリースワークフローの修正

1. **ログの確認**: GitHub Actions のログを確認して、失敗の原因を特定
2. **ワークフローの簡素化**: 複雑な PowerShell スクリプトを分割
3. **テスト**: ローカルで PowerShell スクリプトをテスト

### 代替案

- **GitHub CLI を使用**: `gh release create` コマンドでリリースを作成
- **手動リリース**: 毎回手動でリリースを作成

---

**作成日**: 2026年1月13日  
**ステータス**: 調査中
