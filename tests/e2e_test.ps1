# E2E Test Script for KeyboardRemapperR
# This script tests the QUICKSTART scenario

$ErrorActionPreference = "Stop"
$TestsPassed = 0
$TestsFailed = 0

# Colors for output
function Write-TestResult {
    param(
        [string]$TestName,
        [bool]$Passed,
        [string]$Message = ""
    )
    
    if ($Passed) {
        Write-Host "✓ PASS: $TestName" -ForegroundColor Green
        $script:TestsPassed++
    } else {
        Write-Host "✗ FAIL: $TestName" -ForegroundColor Red
        if ($Message) {
            Write-Host "  Error: $Message" -ForegroundColor Red
        }
        $script:TestsFailed++
    }
}

# Get the binary path
$BinaryPath = "target\release\keyboard-remapper-r.exe"

if (-not (Test-Path $BinaryPath)) {
    Write-Host "Error: Binary not found at $BinaryPath" -ForegroundColor Red
    exit 1
}

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "KeyboardRemapperR E2E Tests" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: List devices (initial state)
Write-Host "[Test 1] List devices (initial state)" -ForegroundColor Yellow
try {
    $output = & $BinaryPath list 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Devices:") {
        Write-TestResult "List devices command" $true
    } else {
        Write-TestResult "List devices command" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "List devices command" $false $_.Exception.Message
}

# Test 2: Set key mapping (remap mode)
Write-Host "`n[Test 2] Set key mapping (remap mode)" -ForegroundColor Yellow
try {
    $output = & $BinaryPath set "04FE:0021" "CapsLock" "LCtrl" "--mode" "remap" 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Mapping set successfully") {
        Write-TestResult "Set remap mapping" $true
    } else {
        Write-TestResult "Set remap mapping" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Set remap mapping" $false $_.Exception.Message
}

# Test 3: Set key mapping (swap mode)
Write-Host "`n[Test 3] Set key mapping (swap mode)" -ForegroundColor Yellow
try {
    $output1 = & $BinaryPath set "04FE:0021" "CapsLock" "LCtrl" "--mode" "swap" 2>&1 | Out-String
    $exitCode1 = $LASTEXITCODE
    
    $output2 = & $BinaryPath set "04FE:0021" "LCtrl" "CapsLock" "--mode" "swap" 2>&1 | Out-String
    $exitCode2 = $LASTEXITCODE
    
    if ($exitCode1 -eq 0 -and $exitCode2 -eq 0 -and $output1 -match "Mapping set successfully" -and $output2 -match "Mapping set successfully") {
        Write-TestResult "Set swap mapping" $true
    } else {
        Write-TestResult "Set swap mapping" $false "Exit codes: $exitCode1, $exitCode2"
    }
} catch {
    Write-TestResult "Set swap mapping" $false $_.Exception.Message
}

# Test 4: Show device configuration
Write-Host "`n[Test 4] Show device configuration" -ForegroundColor Yellow
try {
    $output = & $BinaryPath show "04FE:0021" 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Device:") {
        Write-TestResult "Show device config" $true
    } else {
        Write-TestResult "Show device config" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Show device config" $false $_.Exception.Message
}

# Test 5: Save configuration
Write-Host "`n[Test 5] Save configuration" -ForegroundColor Yellow
$TestConfigPath = "test_config.json"
try {
    $output = & $BinaryPath save "--output" $TestConfigPath 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and (Test-Path $TestConfigPath)) {
        $configContent = Get-Content $TestConfigPath -Raw
        if ($configContent -match "devices") {
            Write-TestResult "Save configuration" $true
        } else {
            Write-TestResult "Save configuration" $false "Invalid JSON content"
        }
    } else {
        Write-TestResult "Save configuration" $false "Exit code: $exitCode or file not created"
    }
} catch {
    Write-TestResult "Save configuration" $false $_.Exception.Message
}

# Test 6: Load configuration
Write-Host "`n[Test 6] Load configuration" -ForegroundColor Yellow
try {
    if (Test-Path $TestConfigPath) {
        $output = & $BinaryPath load "--input" $TestConfigPath 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0 -and $output -match "Configuration loaded successfully") {
            Write-TestResult "Load configuration" $true
        } else {
            Write-TestResult "Load configuration" $false "Exit code: $exitCode, Output: $output"
        }
    } else {
        Write-TestResult "Load configuration" $false "Test config file not found"
    }
} catch {
    Write-TestResult "Load configuration" $false $_.Exception.Message
}

# Test 7: Remove mapping
Write-Host "`n[Test 7] Remove mapping" -ForegroundColor Yellow
try {
    $output = & $BinaryPath remove "04FE:0021" "CapsLock" 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Mapping removed successfully") {
        Write-TestResult "Remove mapping" $true
    } else {
        Write-TestResult "Remove mapping" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Remove mapping" $false $_.Exception.Message
}

# Test 8: Invalid command error handling
Write-Host "`n[Test 8] Invalid command error handling" -ForegroundColor Yellow
try {
    $output = & $BinaryPath invalid-command 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -ne 0) {
        Write-TestResult "Invalid command handling" $true
    } else {
        Write-TestResult "Invalid command handling" $false "Should return non-zero exit code"
    }
} catch {
    # Exception is expected for invalid commands
    Write-TestResult "Invalid command handling" $true
}

# Cleanup
if (Test-Path $TestConfigPath) {
    Remove-Item $TestConfigPath -Force
}

# Summary
Write-Host "`n==================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Passed: $TestsPassed" -ForegroundColor Green
Write-Host "Failed: $TestsFailed" -ForegroundColor Red
Write-Host "Total:  $($TestsPassed + $TestsFailed)" -ForegroundColor White

if ($TestsFailed -eq 0) {
    Write-Host "`n✓ All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n✗ Some tests failed!" -ForegroundColor Red
    exit 1
}
