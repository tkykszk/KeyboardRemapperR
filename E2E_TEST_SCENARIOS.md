# E2E テストシナリオ

## 概要

QUICKSTART のユースケースに基づいた E2E テストを実装します。

## テストシナリオ

### シナリオ 1: デバイス一覧表示

**目的**: `list` コマンドでキーボードデバイスを一覧表示できることを確認

**手順**:
1. `keyboard-remapper-r list` を実行
2. 出力に "Devices:" が含まれることを確認
3. 終了コードが 0 であることを確認

**期待結果**:
```
Devices:
  [No devices configured yet]
```

### シナリオ 2: キーマッピング設定（リマップ）

**目的**: `set` コマンドでキーマッピングを設定できることを確認

**手順**:
1. `keyboard-remapper-r set 04FE:0021 CapsLock LCtrl --mode remap` を実行
2. 終了コードが 0 であることを確認
3. 出力に "Mapping set successfully" が含まれることを確認

**期待結果**:
```
Mapping set successfully: CapsLock -> LCtrl (remap) for device 04FE:0021
```

### シナリオ 3: キーマッピング設定（スワップ）

**目的**: `set` コマンドでキーのスワップを設定できることを確認

**手順**:
1. `keyboard-remapper-r set 04FE:0021 CapsLock LCtrl --mode swap` を実行
2. `keyboard-remapper-r set 04FE:0021 LCtrl CapsLock --mode swap` を実行
3. 終了コードが 0 であることを確認

**期待結果**:
```
Mapping set successfully: CapsLock <-> LCtrl (swap) for device 04FE:0021
```

### シナリオ 4: デバイス設定の表示

**目的**: `show` コマンドで特定デバイスの設定を表示できることを確認

**手順**:
1. `keyboard-remapper-r show 04FE:0021` を実行
2. 出力に設定したマッピングが含まれることを確認
3. 終了コードが 0 であることを確認

**期待結果**:
```
Device: 04FE:0021
Mappings:
  CapsLock -> LCtrl (swap)
  LCtrl -> CapsLock (swap)
```

### シナリオ 5: 設定の保存

**目的**: `save` コマンドで設定を JSON ファイルに保存できることを確認

**手順**:
1. `keyboard-remapper-r save --output test_config.json` を実行
2. 終了コードが 0 であることを確認
3. `test_config.json` ファイルが存在することを確認
4. JSON ファイルの内容が正しいことを確認

**期待結果**:
```json
{
  "devices": {
    "04FE:0021": {
      "mappings": [
        {
          "from": "CapsLock",
          "to": "LCtrl",
          "type": "Swap"
        },
        {
          "from": "LCtrl",
          "to": "CapsLock",
          "type": "Swap"
        }
      ]
    }
  }
}
```

### シナリオ 6: 設定の読み込み

**目的**: `load` コマンドで JSON ファイルから設定を読み込めることを確認

**手順**:
1. `keyboard-remapper-r load --input test_config.json` を実行
2. 終了コードが 0 であることを確認
3. 出力に "Configuration loaded successfully" が含まれることを確認

**期待結果**:
```
Configuration loaded successfully from test_config.json
```

### シナリオ 7: マッピングの削除

**目的**: `remove` コマンドでキーマッピングを削除できることを確認

**手順**:
1. `keyboard-remapper-r remove 04FE:0021 CapsLock` を実行
2. 終了コードが 0 であることを確認
3. 出力に "Mapping removed successfully" が含まれることを確認

**期待結果**:
```
Mapping removed successfully: CapsLock for device 04FE:0021
```

### シナリオ 8: 無効なコマンドのエラーハンドリング

**目的**: 無効なコマンドが適切にエラーを返すことを確認

**手順**:
1. `keyboard-remapper-r invalid-command` を実行
2. 終了コードが 0 以外であることを確認
3. エラーメッセージが表示されることを確認

**期待結果**:
```
error: unrecognized subcommand 'invalid-command'
```

## テスト実行環境

- **OS**: Windows Server 2022
- **Rust**: 最新安定版
- **実行方法**: PowerShell スクリプト

## 成功基準

- すべてのシナリオが成功すること
- 終了コードが期待通りであること
- 出力メッセージが期待通りであること
- JSON ファイルの内容が正しいこと
