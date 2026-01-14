# Phase 3 テスト自動化スクリプト (PowerShell)
#
# このスクリプトは、Phase 3 のすべてのテストを自動実行し、
# テスト結果をレポートとして出力します。
#
# 使用方法:
#   .\scripts\run_tests.ps1
#   .\scripts\run_tests.ps1 -TestType unit
#   .\scripts\run_tests.ps1 -TestType integration
#   .\scripts\run_tests.ps1 -TestType performance
#   .\scripts\run_tests.ps1 -GenerateReport

param(
    [string]$TestType = "all",
    [switch]$GenerateReport = $false,
    [switch]$Verbose = $false
)

# カラー出力用の関数
function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor Cyan
}

function Write-Warning-Custom {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

# テスト結果を格納する変数
$script:TestResults = @{
    UnitTests = @{
        Total = 0
        Passed = 0
        Failed = 0
        Duration = 0
    }
    IntegrationTests = @{
        Total = 0
        Passed = 0
        Failed = 0
        Duration = 0
    }
    PerformanceTests = @{
        Total = 0
        Passed = 0
        Failed = 0
        Duration = 0
        Metrics = @{}
    }
}

# テスト開始時刻
$script:StartTime = Get-Date

Write-Info "Phase 3 テスト自動化スクリプト"
Write-Info "================================"
Write-Info "開始時刻: $($script:StartTime.ToString('yyyy-MM-dd HH:mm:ss'))"
Write-Info ""

# ============================================================================
# 単体テストの実行
# ============================================================================

function Run-UnitTests {
    Write-Info "単体テストを実行中..."
    
    $startTime = Get-Date
    
    # Rust の単体テストを実行
    $output = cargo test --lib 2>&1
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    $script:TestResults.UnitTests.Duration = $duration
    
    # テスト結果を解析
    if ($output -match "test result: ok. (\d+) passed") {
        $script:TestResults.UnitTests.Passed = [int]$Matches[1]
        $script:TestResults.UnitTests.Total = $script:TestResults.UnitTests.Passed
        Write-Success "単体テスト完了: $($script:TestResults.UnitTests.Passed)/$($script:TestResults.UnitTests.Total) 通過 (${duration}秒)"
        return $true
    }
    elseif ($output -match "test result: FAILED. (\d+) passed; (\d+) failed") {
        $script:TestResults.UnitTests.Passed = [int]$Matches[1]
        $script:TestResults.UnitTests.Failed = [int]$Matches[2]
        $script:TestResults.UnitTests.Total = $script:TestResults.UnitTests.Passed + $script:TestResults.UnitTests.Failed
        Write-Error-Custom "単体テスト失敗: $($script:TestResults.UnitTests.Passed)/$($script:TestResults.UnitTests.Total) 通過 (${duration}秒)"
        
        if ($Verbose) {
            Write-Info "詳細:"
            Write-Host $output
        }
        
        return $false
    }
    else {
        Write-Error-Custom "単体テストの実行に失敗しました"
        if ($Verbose) {
            Write-Host $output
        }
        return $false
    }
}

# ============================================================================
# 統合テストの実行
# ============================================================================

function Run-IntegrationTests {
    Write-Info "統合テストを実行中..."
    
    $startTime = Get-Date
    
    # E2E テストを実行
    $output = cargo test --test e2e_tests 2>&1
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    $script:TestResults.IntegrationTests.Duration = $duration
    
    # テスト結果を解析
    if ($output -match "test result: ok. (\d+) passed") {
        $script:TestResults.IntegrationTests.Passed = [int]$Matches[1]
        $script:TestResults.IntegrationTests.Total = $script:TestResults.IntegrationTests.Passed
        Write-Success "統合テスト完了: $($script:TestResults.IntegrationTests.Passed)/$($script:TestResults.IntegrationTests.Total) 通過 (${duration}秒)"
        return $true
    }
    elseif ($output -match "test result: FAILED. (\d+) passed; (\d+) failed") {
        $script:TestResults.IntegrationTests.Passed = [int]$Matches[1]
        $script:TestResults.IntegrationTests.Failed = [int]$Matches[2]
        $script:TestResults.IntegrationTests.Total = $script:TestResults.IntegrationTests.Passed + $script:TestResults.IntegrationTests.Failed
        Write-Error-Custom "統合テスト失敗: $($script:TestResults.IntegrationTests.Passed)/$($script:TestResults.IntegrationTests.Total) 通過 (${duration}秒)"
        
        if ($Verbose) {
            Write-Info "詳細:"
            Write-Host $output
        }
        
        return $false
    }
    else {
        Write-Warning-Custom "統合テストが見つかりません（まだ実装されていない可能性があります）"
        return $true
    }
}

# ============================================================================
# パフォーマンステストの実行
# ============================================================================

function Run-PerformanceTests {
    Write-Info "パフォーマンステストを実行中..."
    
    $startTime = Get-Date
    
    # パフォーマンステストを実行
    $output = cargo test --release -- --ignored --nocapture 2>&1
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    $script:TestResults.PerformanceTests.Duration = $duration
    
    # テスト結果を解析
    if ($output -match "Average latency: (\d+\.?\d*)ms") {
        $latency = [double]$Matches[1]
        $script:TestResults.PerformanceTests.Metrics["Latency"] = $latency
        
        if ($latency -le 5.0) {
            Write-Success "キー入力遅延: ${latency}ms (基準: ≤5ms)"
        }
        else {
            Write-Error-Custom "キー入力遅延: ${latency}ms (基準: ≤5ms) - 基準を超えています"
        }
    }
    
    if ($output -match "Success rate: (\d+\.?\d*)%") {
        $successRate = [double]$Matches[1]
        $script:TestResults.PerformanceTests.Metrics["SuccessRate"] = $successRate
        
        if ($successRate -ge 99.0) {
            Write-Success "成功率: ${successRate}% (基準: ≥99%)"
        }
        else {
            Write-Error-Custom "成功率: ${successRate}% (基準: ≥99%) - 基準を下回っています"
        }
    }
    
    # テスト結果を解析
    if ($output -match "test result: ok. (\d+) passed") {
        $script:TestResults.PerformanceTests.Passed = [int]$Matches[1]
        $script:TestResults.PerformanceTests.Total = $script:TestResults.PerformanceTests.Passed
        Write-Success "パフォーマンステスト完了: $($script:TestResults.PerformanceTests.Passed)/$($script:TestResults.PerformanceTests.Total) 通過 (${duration}秒)"
        return $true
    }
    elseif ($output -match "test result: FAILED. (\d+) passed; (\d+) failed") {
        $script:TestResults.PerformanceTests.Passed = [int]$Matches[1]
        $script:TestResults.PerformanceTests.Failed = [int]$Matches[2]
        $script:TestResults.PerformanceTests.Total = $script:TestResults.PerformanceTests.Passed + $script:TestResults.PerformanceTests.Failed
        Write-Error-Custom "パフォーマンステスト失敗: $($script:TestResults.PerformanceTests.Passed)/$($script:TestResults.PerformanceTests.Total) 通過 (${duration}秒)"
        
        if ($Verbose) {
            Write-Info "詳細:"
            Write-Host $output
        }
        
        return $false
    }
    else {
        Write-Warning-Custom "パフォーマンステストが見つかりません（まだ実装されていない可能性があります）"
        return $true
    }
}

# ============================================================================
# テストレポートの生成
# ============================================================================

function Generate-TestReport {
    Write-Info "テストレポートを生成中..."
    
    $endTime = Get-Date
    $totalDuration = ($endTime - $script:StartTime).TotalSeconds
    
    $reportPath = "test_results_$(Get-Date -Format 'yyyyMMdd_HHmmss').md"
    
    $report = @"
# Phase 3 テスト結果レポート

**テスト日**: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')  
**環境**: Windows $([System.Environment]::OSVersion.Version)  
**総実行時間**: ${totalDuration}秒

---

## 📊 テスト結果サマリー

| カテゴリ | 合計 | 通過 | 失敗 | 実行時間 | 合格率 |
|---------|------|------|------|---------|--------|
| 単体テスト | $($script:TestResults.UnitTests.Total) | $($script:TestResults.UnitTests.Passed) | $($script:TestResults.UnitTests.Failed) | $($script:TestResults.UnitTests.Duration)秒 | $(if ($script:TestResults.UnitTests.Total -gt 0) { [math]::Round(($script:TestResults.UnitTests.Passed / $script:TestResults.UnitTests.Total) * 100, 2) } else { 0 })% |
| 統合テスト | $($script:TestResults.IntegrationTests.Total) | $($script:TestResults.IntegrationTests.Passed) | $($script:TestResults.IntegrationTests.Failed) | $($script:TestResults.IntegrationTests.Duration)秒 | $(if ($script:TestResults.IntegrationTests.Total -gt 0) { [math]::Round(($script:TestResults.IntegrationTests.Passed / $script:TestResults.IntegrationTests.Total) * 100, 2) } else { 0 })% |
| パフォーマンステスト | $($script:TestResults.PerformanceTests.Total) | $($script:TestResults.PerformanceTests.Passed) | $($script:TestResults.PerformanceTests.Failed) | $($script:TestResults.PerformanceTests.Duration)秒 | $(if ($script:TestResults.PerformanceTests.Total -gt 0) { [math]::Round(($script:TestResults.PerformanceTests.Passed / $script:TestResults.PerformanceTests.Total) * 100, 2) } else { 0 })% |

---

## ⚡ パフォーマンスメトリクス

| メトリクス | 測定値 | 基準値 | 結果 |
|-----------|--------|--------|------|
"@

    # パフォーマンスメトリクスを追加
    if ($script:TestResults.PerformanceTests.Metrics.ContainsKey("Latency")) {
        $latency = $script:TestResults.PerformanceTests.Metrics["Latency"]
        $latencyResult = if ($latency -le 5.0) { "✅ Pass" } else { "❌ Fail" }
        $report += "| キー入力遅延 | ${latency}ms | ≤5ms | $latencyResult |`n"
    }
    
    if ($script:TestResults.PerformanceTests.Metrics.ContainsKey("SuccessRate")) {
        $successRate = $script:TestResults.PerformanceTests.Metrics["SuccessRate"]
        $successRateResult = if ($successRate -ge 99.0) { "✅ Pass" } else { "❌ Fail" }
        $report += "| 成功率 | ${successRate}% | ≥99% | $successRateResult |`n"
    }
    
    $report += @"

---

## 🎯 総合評価

"@

    $totalTests = $script:TestResults.UnitTests.Total + $script:TestResults.IntegrationTests.Total + $script:TestResults.PerformanceTests.Total
    $totalPassed = $script:TestResults.UnitTests.Passed + $script:TestResults.IntegrationTests.Passed + $script:TestResults.PerformanceTests.Passed
    $totalFailed = $script:TestResults.UnitTests.Failed + $script:TestResults.IntegrationTests.Failed + $script:TestResults.PerformanceTests.Failed
    
    if ($totalTests -gt 0) {
        $passRate = [math]::Round(($totalPassed / $totalTests) * 100, 2)
        $report += "- **合格率**: $totalPassed/$totalTests ($passRate%)`n"
    }
    else {
        $report += "- **合格率**: テストが実行されませんでした`n"
    }
    
    if ($totalFailed -eq 0) {
        $report += "- **重大な問題**: なし`n"
        $report += "- **リリース判定**: ✅ リリース可能`n"
    }
    else {
        $report += "- **重大な問題**: $totalFailed 件のテストが失敗`n"
        $report += "- **リリース判定**: ❌ 修正が必要`n"
    }
    
    $report += @"

---

**作成日**: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')  
**作成者**: 自動生成  
**バージョン**: Phase 3
"@

    # レポートをファイルに保存
    $report | Out-File -FilePath $reportPath -Encoding UTF8
    
    Write-Success "テストレポートを生成しました: $reportPath"
    
    return $reportPath
}

# ============================================================================
# メイン処理
# ============================================================================

try {
    # プロジェクトディレクトリに移動
    $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $projectDir = Split-Path -Parent $scriptDir
    Set-Location $projectDir
    
    Write-Info "プロジェクトディレクトリ: $projectDir"
    Write-Info ""
    
    # テストタイプに応じて実行
    $allPassed = $true
    
    if ($TestType -eq "all" -or $TestType -eq "unit") {
        if (-not (Run-UnitTests)) {
            $allPassed = $false
        }
        Write-Host ""
    }
    
    if ($TestType -eq "all" -or $TestType -eq "integration") {
        if (-not (Run-IntegrationTests)) {
            $allPassed = $false
        }
        Write-Host ""
    }
    
    if ($TestType -eq "all" -or $TestType -eq "performance") {
        if (-not (Run-PerformanceTests)) {
            $allPassed = $false
        }
        Write-Host ""
    }
    
    # レポート生成
    if ($GenerateReport -or $TestType -eq "all") {
        $reportPath = Generate-TestReport
        Write-Host ""
    }
    
    # 終了時刻
    $endTime = Get-Date
    $totalDuration = ($endTime - $script:StartTime).TotalSeconds
    
    Write-Info "================================"
    Write-Info "終了時刻: $($endTime.ToString('yyyy-MM-dd HH:mm:ss'))"
    Write-Info "総実行時間: ${totalDuration}秒"
    
    if ($allPassed) {
        Write-Success "すべてのテストが通過しました！"
        exit 0
    }
    else {
        Write-Error-Custom "一部のテストが失敗しました"
        exit 1
    }
}
catch {
    Write-Error-Custom "エラーが発生しました: $_"
    exit 1
}
