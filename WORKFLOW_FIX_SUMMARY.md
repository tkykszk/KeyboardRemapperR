# GitHub Actions Workflow 修正サマリー

## 修正日時
2026-01-17

## 問題の詳細

### エラーメッセージ
```
Invalid workflow file: .github/workflows/release.yml#L78
You have an error in your yaml syntax on line 78
```

### 原因
`release.yml` の78行目で PowerShell の here-string 構文 `@'...'@` を使用していましたが、YAML パーサーがこの構文を正しく解釈できませんでした。

## 修正内容

### 変更前（78-91行目）
```yaml
# Create sample config
$configContent = @'
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
'@
$configContent | Out-File -FilePath "$packageName\config.sample.json" -Encoding utf8
```

### 変更後（78-92行目）
```yaml
# Create sample config
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
```

### 主な変更点
1. `$configContent = @'...'@` を `@"..."@` に変更
2. 変数への代入を削除し、直接パイプライン処理に変更
3. YAML パーサーが正しく解釈できる形式に修正

## コミット情報

**コミットハッシュ**: `8dbbe63`

**コミットメッセージ**:
```
fix: Fix YAML syntax error in release.yml

- Replace PowerShell here-string syntax (@'...'@) with @"..."@ to avoid YAML parsing issues
- This fixes the 'Invalid workflow file' error on line 78
```

## プッシュ結果

✅ **成功**: `feature/device-detection` ブランチにプッシュ完了

```
To https://github.com/tkykszk/KeyboardRemapperR.git
   bba37ee..8dbbe63  feature/device-detection -> feature/device-detection
```

## 期待される結果

この修正により、以下のワークフローが正常に動作するはずです:

1. **Phase 3 Tests** (test.yml): 引き続き正常動作
2. **Release** (release.yml): YAML 構文エラーが解消され、タグプッシュ時に正常動作

## 検証方法

次回のコミット・プッシュ時に GitHub Actions が自動実行されます。以下を確認してください:

1. Phase 3 Tests ワークフローが成功すること
2. release.yml ワークフローの YAML 構文エラーが解消されていること

## 補足

`release.yml` ワークフローは `v*` タグがプッシュされた時のみ実行されます。通常のコミット・プッシュでは実行されません。
