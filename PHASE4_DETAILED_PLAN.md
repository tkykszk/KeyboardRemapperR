# Phase 4: サービス化と管理機能 - 詳細計画

**作成日**: 2026年1月14日  
**優先度**: 中  
**見積もり時間**: 10-13時間  
**前提条件**: Phase 1-3 完了

---

## 📋 Phase 4 概要

Phase 4 では、KeyboardRemapperR をバックグラウンドで動作させるためのサービス化と、設定の動的な管理機能を実装します。これにより、ユーザーはアプリケーションを常駐させ、再起動なしで設定を変更できるようになります。

### 主要な目標

1. **バックグラウンド実行**: Windows サービスとして動作
2. **Start/Stop コマンド**: サービスの起動/停止を管理
3. **設定のホットリロード**: 再起動なしで設定変更を反映
4. **ログ機能**: デバッグ用のログ出力

---

## 🎯 タスク一覧

| タスク | 内容 | 見積もり | 依存関係 |
|--------|------|----------|----------|
| **Task 4.1** | バックグラウンド実行の実装 | 3-4時間 | Phase 3 |
| **Task 4.2** | Start/Stop コマンドの実装 | 2-3時間 | Task 4.1 |
| **Task 4.3** | 設定のホットリロードの実装 | 3-4時間 | Task 4.1 |
| **Task 4.4** | ログ機能の実装 | 2-3時間 | なし |
| **合計** | - | **10-14時間** | - |

---

## 📝 Task 4.1: バックグラウンド実行の実装

### 目的

KeyboardRemapperR を Windows サービスとして動作させ、バックグラウンドで常駐できるようにします。

### 実装内容

#### 1. Windows サービスの基本構造

Windows サービスは以下の要素で構成されます:

- **サービスコントロールハンドラ**: サービスの状態を管理
- **サービスメイン**: サービスのメインループ
- **サービス登録**: SCM（Service Control Manager）への登録

#### 2. 実装アプローチ

**オプション A: Windows Service API を直接使用**（推奨）

Rust の `winapi` クレートを使用して、Windows Service API を直接呼び出します。

**利点**:
- 依存関係が少ない
- 完全な制御が可能
- 既存のコードとの統合が容易

**欠点**:
- 実装が複雑
- エラーハンドリングが難しい

**オプション B: `windows-service` クレートを使用**

Rust の `windows-service` クレートを使用して、サービスを実装します。

**利点**:
- 実装が簡単
- エラーハンドリングが容易
- ドキュメントが充実

**欠点**:
- 依存関係が増える
- カスタマイズが制限される

**推奨**: オプション B（`windows-service` クレート）を使用

#### 3. 実装手順

**Step 1: 依存関係の追加**

`Cargo.toml` に以下を追加:

```toml
[dependencies]
windows-service = "0.6"
```

**Step 2: サービス構造体の定義**

```rust
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState,
        ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

define_windows_service!(ffi_service_main, keyboard_remapper_service_main);

fn keyboard_remapper_service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(arguments) {
        // Log error
        eprintln!("Service error: {}", e);
    }
}
```

**Step 3: サービスコントロールハンドラの実装**

```rust
fn run_service(_arguments: Vec<OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register("KeyboardRemapperR", event_handler)?;

    // Tell Windows that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Run the main loop
    run_main_loop(shutdown_rx)?;

    // Tell Windows that service is stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}
```

**Step 4: メインループの実装**

```rust
fn run_main_loop(shutdown_rx: std::sync::mpsc::Receiver<()>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config_or_default();
    
    // Install keyboard hook
    unsafe { install_keyboard_hook()? };
    
    // Run message loop in a separate thread
    let handle = std::thread::spawn(move || {
        unsafe { RawInputHandler::run_message_loop(config) }
    });
    
    // Wait for shutdown signal
    shutdown_rx.recv()?;
    
    // Cleanup
    unsafe { uninstall_keyboard_hook(); }
    
    // Wait for message loop to finish
    handle.join().unwrap()?;
    
    Ok(())
}
```

**Step 5: サービスのエントリーポイント**

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if running as a service
    if std::env::args().any(|arg| arg == "--service") {
        // Run as Windows service
        service_dispatcher::start("KeyboardRemapperR", ffi_service_main)?;
    } else {
        // Run as console application
        run_console_app()?;
    }
    Ok(())
}
```

#### 4. サービスのインストール

**PowerShell スクリプト** (`install_service.ps1`):

```powershell
# Requires administrator privileges
$serviceName = "KeyboardRemapperR"
$exePath = "$PSScriptRoot\keyboard-remapper-r.exe"
$displayName = "Keyboard Remapper R"
$description = "Device-specific keyboard remapper for Windows"

# Check if service exists
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($service) {
    Write-Host "Service already exists. Stopping and removing..."
    Stop-Service -Name $serviceName -Force
    sc.exe delete $serviceName
    Start-Sleep -Seconds 2
}

# Create service
Write-Host "Creating service..."
sc.exe create $serviceName binPath= "$exePath --service" start= auto DisplayName= "$displayName"
sc.exe description $serviceName "$description"

# Start service
Write-Host "Starting service..."
Start-Service -Name $serviceName

Write-Host "Service installed and started successfully."
```

**PowerShell スクリプト** (`uninstall_service.ps1`):

```powershell
# Requires administrator privileges
$serviceName = "KeyboardRemapperR"

# Check if service exists
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($service) {
    Write-Host "Stopping service..."
    Stop-Service -Name $serviceName -Force
    
    Write-Host "Removing service..."
    sc.exe delete $serviceName
    
    Write-Host "Service uninstalled successfully."
} else {
    Write-Host "Service not found."
}
```

#### 5. テスト

**単体テスト**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_installation() {
        // Test service installation logic
    }

    #[test]
    fn test_service_control_handler() {
        // Test service control handler
    }
}
```

**統合テスト**:

```powershell
# Test service installation
.\install_service.ps1

# Check service status
Get-Service -Name KeyboardRemapperR

# Test service start/stop
Stop-Service -Name KeyboardRemapperR
Start-Service -Name KeyboardRemapperR

# Test service uninstallation
.\uninstall_service.ps1
```

### 成果物

- ✅ Windows サービスとして動作する実装
- ✅ サービスのインストール/アンインストールスクリプト
- ✅ サービスコントロールハンドラ
- ✅ 単体テストと統合テスト

### 見積もり時間

**3-4時間**

---

## 📝 Task 4.2: Start/Stop コマンドの実装

### 目的

CLI から Windows サービスを起動/停止できるようにします。

### 実装内容

#### 1. Start コマンドの更新

現在の `Start` コマンドは、フォアグラウンドで実行されます。これをバックグラウンドサービスとして起動するように変更します。

```rust
Commands::Start => {
    println!("Starting KeyboardRemapperR service...");
    
    // Check if service is already running
    if is_service_running("KeyboardRemapperR")? {
        println!("Service is already running.");
        return Ok(());
    }
    
    // Start service
    start_service("KeyboardRemapperR")?;
    
    println!("Service started successfully.");
    println!("Use 'keyboard-remapper-r stop' to stop the service.");
}
```

#### 2. Stop コマンドの実装

```rust
Commands::Stop => {
    println!("Stopping KeyboardRemapperR service...");
    
    // Check if service is running
    if !is_service_running("KeyboardRemapperR")? {
        println!("Service is not running.");
        return Ok(());
    }
    
    // Stop service
    stop_service("KeyboardRemapperR")?;
    
    println!("Service stopped successfully.");
}
```

#### 3. ヘルパー関数の実装

```rust
fn is_service_running(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let output = Command::new("sc")
        .args(&["query", service_name])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains("RUNNING"))
}

fn start_service(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let output = Command::new("sc")
        .args(&["start", service_name])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to start service: {}", stderr).into());
    }
    
    Ok(())
}

fn stop_service(service_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    let output = Command::new("sc")
        .args(&["stop", service_name])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stop service: {}", stderr).into());
    }
    
    Ok(())
}
```

#### 4. Status コマンドの追加

サービスの状態を確認するコマンドを追加します。

```rust
Commands::Status => {
    println!("Checking KeyboardRemapperR service status...");
    
    if is_service_running("KeyboardRemapperR")? {
        println!("Service is running.");
    } else {
        println!("Service is not running.");
    }
}
```

#### 5. テスト

**単体テスト**:

```rust
#[test]
fn test_is_service_running() {
    // Test service status check
}

#[test]
fn test_start_service() {
    // Test service start
}

#[test]
fn test_stop_service() {
    // Test service stop
}
```

### 成果物

- ✅ Start コマンドの更新（サービス起動）
- ✅ Stop コマンドの実装（サービス停止）
- ✅ Status コマンドの追加（サービス状態確認）
- ✅ ヘルパー関数の実装
- ✅ 単体テスト

### 見積もり時間

**2-3時間**

---

## 📝 Task 4.3: 設定のホットリロードの実装

### 目的

サービスを再起動せずに、設定ファイルの変更を動的に反映できるようにします。

### 実装内容

#### 1. ファイル監視の実装

`notify` クレートを使用して、設定ファイルの変更を監視します。

**依存関係の追加**:

```toml
[dependencies]
notify = "6.0"
```

**ファイル監視の実装**:

```rust
use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use std::sync::mpsc::channel;
use std::time::Duration;

fn watch_config_file(config_path: &Path) -> NotifyResult<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(config_path, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                match event {
                    Ok(notify::Event { kind: notify::EventKind::Modify(_), .. }) => {
                        println!("Config file changed, reloading...");
                        reload_config()?;
                    }
                    _ => {}
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Continue loop
            }
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
```

#### 2. 設定のリロード

```rust
fn reload_config() -> Result<(), Box<dyn std::error::Error>> {
    // Load new config
    let new_config = load_config_or_default();
    
    // Update global config
    unsafe {
        if let Some(handler) = &mut GLOBAL_HANDLER {
            handler.config = new_config;
            println!("Configuration reloaded successfully.");
        }
    }
    
    // Update suppressed keys based on new config
    update_suppressed_keys()?;
    
    Ok(())
}

fn update_suppressed_keys() -> Result<(), Box<dyn std::error::Error>> {
    // Clear current suppressed keys
    unsafe {
        if let Some(keys) = &mut SUPPRESSED_KEYS {
            keys.clear();
        }
    }
    
    // Add suppressed keys from new config
    unsafe {
        if let Some(handler) = &GLOBAL_HANDLER {
            for device in &handler.config.devices {
                for mapping in &device.mappings {
                    if let Some(vk) = key_name_to_vk(&mapping.from) {
                        add_suppressed_key(vk);
                    }
                }
            }
        }
    }
    
    Ok(())
}
```

#### 3. Reload コマンドの追加

手動でリロードするコマンドを追加します。

```rust
Commands::Reload => {
    println!("Reloading configuration...");
    
    // Check if service is running
    if !is_service_running("KeyboardRemapperR")? {
        println!("Service is not running. Configuration will be loaded on next start.");
        return Ok(());
    }
    
    // Reload config
    reload_config()?;
    
    println!("Configuration reloaded successfully.");
}
```

#### 4. 自動リロードの統合

サービスメインループに自動リロードを統合します。

```rust
fn run_main_loop(shutdown_rx: std::sync::mpsc::Receiver<()>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config_or_default();
    
    // Install keyboard hook
    unsafe { install_keyboard_hook()? };
    
    // Start config file watcher
    let config_path = PathBuf::from("config.json");
    let watcher_handle = std::thread::spawn(move || {
        watch_config_file(&config_path)
    });
    
    // Run message loop in a separate thread
    let handle = std::thread::spawn(move || {
        unsafe { RawInputHandler::run_message_loop(config) }
    });
    
    // Wait for shutdown signal
    shutdown_rx.recv()?;
    
    // Cleanup
    unsafe { uninstall_keyboard_hook(); }
    
    // Wait for threads to finish
    handle.join().unwrap()?;
    // Note: watcher_handle will be terminated when process exits
    
    Ok(())
}
```

#### 5. テスト

**単体テスト**:

```rust
#[test]
fn test_reload_config() {
    // Test config reload
}

#[test]
fn test_update_suppressed_keys() {
    // Test suppressed keys update
}
```

**統合テスト**:

```powershell
# Start service
keyboard-remapper-r.exe start

# Modify config.json
# ...

# Wait for auto-reload or manually reload
keyboard-remapper-r.exe reload

# Verify new config is applied
keyboard-remapper-r.exe show 04FE:0021
```

### 成果物

- ✅ ファイル監視の実装
- ✅ 設定のリロード機能
- ✅ Reload コマンドの追加
- ✅ 自動リロードの統合
- ✅ 単体テストと統合テスト

### 見積もり時間

**3-4時間**

---

## 📝 Task 4.4: ログ機能の実装

### 目的

デバッグとトラブルシューティングのために、ログ機能を実装します。

### 実装内容

#### 1. ログライブラリの選択

`log` と `env_logger` クレートを使用します。

**依存関係の追加**:

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

#### 2. ログの初期化

```rust
use log::{info, warn, error, debug};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    info!("KeyboardRemapperR starting...");
    
    // ...
    
    Ok(())
}
```

#### 3. ログの追加

主要な箇所にログを追加します。

```rust
// Device detection
info!("Detected {} keyboard devices", devices.len());
for device in &devices {
    debug!("Device: {} - {}", device.device_id, device.device_name);
}

// Mapping application
info!("Applying mapping: {} -> {} on device {}", from, to, device_id);

// Key remapping
debug!("Key {} pressed on device {}", key_name, device_id);
debug!("Remapping {} to {}", key_name, mapped_key);

// Service events
info!("Service started");
info!("Service stopped");
warn!("Failed to load config: {}", error);
error!("Fatal error: {}", error);
```

#### 4. ログファイルの出力

ログをファイルに出力するように設定します。

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("keyboard-remapper-r.log")?;
    
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(log::LevelFilter::Info)
        .init();
    
    Ok(())
}
```

#### 5. ログレベルの設定

環境変数でログレベルを設定できるようにします。

```powershell
# Set log level to debug
$env:RUST_LOG="debug"
keyboard-remapper-r.exe start

# Set log level to info
$env:RUST_LOG="info"
keyboard-remapper-r.exe start
```

#### 6. Logs コマンドの追加

ログファイルを表示するコマンドを追加します。

```rust
Commands::Logs { lines } => {
    println!("Showing last {} lines of log file...", lines);
    
    let log_file = PathBuf::from("keyboard-remapper-r.log");
    if !log_file.exists() {
        println!("Log file not found.");
        return Ok(());
    }
    
    // Read last N lines
    let content = std::fs::read_to_string(&log_file)?;
    let lines_vec: Vec<&str> = content.lines().collect();
    let start = lines_vec.len().saturating_sub(lines);
    
    for line in &lines_vec[start..] {
        println!("{}", line);
    }
}
```

#### 7. テスト

**単体テスト**:

```rust
#[test]
fn test_logger_initialization() {
    // Test logger initialization
}
```

### 成果物

- ✅ ログライブラリの統合
- ✅ 主要箇所へのログ追加
- ✅ ログファイルへの出力
- ✅ ログレベルの設定
- ✅ Logs コマンドの追加
- ✅ 単体テスト

### 見積もり時間

**2-3時間**

---

## 🧪 Phase 4 テスト計画

### 単体テスト

- ✅ サービスインストールのテスト
- ✅ サービスコントロールハンドラのテスト
- ✅ Start/Stop コマンドのテスト
- ✅ 設定リロードのテスト
- ✅ ログ機能のテスト

### 統合テスト

- ✅ サービスのインストール/アンインストール
- ✅ サービスの起動/停止
- ✅ 設定ファイルの変更と自動リロード
- ✅ ログファイルの出力確認

### パフォーマンステスト

- ✅ サービス起動時間の測定
- ✅ 設定リロード時間の測定
- ✅ ログ出力のオーバーヘッド測定

---

## 📊 Phase 4 完了基準

### 必須項目

- ✅ Windows サービスとして動作する
- ✅ Start/Stop コマンドでサービスを制御できる
- ✅ 設定ファイルの変更が自動的に反映される
- ✅ ログファイルに動作ログが出力される
- ✅ すべてのテストが通過する

### パフォーマンス基準

- ✅ サービス起動時間: ≤3秒
- ✅ 設定リロード時間: ≤1秒
- ✅ ログ出力のオーバーヘッド: ≤1%

---

## 🚀 Phase 4 完了後

### v0.2.0-beta1 リリース

Phase 4 完了後、**v0.2.0-beta1** をリリースします。

**実装済み機能**:
- ✅ Phase 1-3 の全機能
- ✅ Windows サービス化
- ✅ Start/Stop コマンド
- ✅ 設定のホットリロード
- ✅ ログ機能

**制限事項**:
- GUI は未実装（Phase 6）
- 修飾キー付きリマップは未実装（Phase 6）

### 次のステップ

**Phase 5: テストとドキュメント**
- 統合テストの作成
- 完全版E2Eテストの修正
- ユーザーガイド
- 開発者ドキュメント

**Phase 6: 高度な機能**
- 修飾キー付きリマップ
- マクロ機能
- プロファイル機能
- GUI版

---

## 📝 実装の注意点

### 1. 管理者権限

Windows サービスのインストール/アンインストールには管理者権限が必要です。

```powershell
# Run as administrator
Start-Process powershell -Verb RunAs
```

### 2. サービスのデバッグ

サービスのデバッグは、コンソールアプリケーションとして実行することで行います。

```bash
# Run as console app (not as service)
keyboard-remapper-r.exe start --console
```

### 3. 設定ファイルのパス

サービスとして実行する場合、設定ファイルのパスは絶対パスを使用します。

```rust
let config_path = std::env::current_exe()?
    .parent()
    .unwrap()
    .join("config.json");
```

### 4. エラーハンドリング

サービスとして実行する場合、エラーをログファイルに出力します。

```rust
if let Err(e) = run_service(arguments) {
    error!("Service error: {}", e);
}
```

---

## 📈 見積もり時間の内訳

| タスク | 見積もり | 実装 | テスト | ドキュメント |
|--------|----------|------|--------|-------------|
| Task 4.1 | 3-4時間 | 2-3時間 | 0.5-1時間 | 0.5時間 |
| Task 4.2 | 2-3時間 | 1-2時間 | 0.5-1時間 | 0.5時間 |
| Task 4.3 | 3-4時間 | 2-3時間 | 0.5-1時間 | 0.5時間 |
| Task 4.4 | 2-3時間 | 1-2時間 | 0.5-1時間 | 0.5時間 |
| **合計** | **10-14時間** | **6-10時間** | **2-4時間** | **2時間** |

---

**作成日**: 2026年1月14日  
**作成者**: tkykszk  
**バージョン**: Phase 4 詳細計画
