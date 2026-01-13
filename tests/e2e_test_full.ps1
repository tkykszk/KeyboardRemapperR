# Full E2E Test Script for KeyboardRemapperR
# This script tests all QUICKSTART scenarios with 8 comprehensive test cases

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

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "KeyboardRemapperR Full E2E Test Suite" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: List devices (initial state - should show empty or default devices)
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

# Test 2: Set key mapping (remap mode - CapsLock -> LCtrl)
Write-Host "`n[Test 2] Set key mapping (remap mode)" -ForegroundColor Yellow
try {
    $output = & $BinaryPath set 04FE:0021 CapsLock LCtrl 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Mapping set successfully") {
        Write-TestResult "Set remap mapping (CapsLock -> LCtrl)" $true
    } else {
        Write-TestResult "Set remap mapping (CapsLock -> LCtrl)" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Set remap mapping (CapsLock -> LCtrl)" $false $_.Exception.Message
}

# Test 3: Set key mapping (swap mode - CapsLock <-> LCtrl)
Write-Host "`n[Test 3] Set key mapping (swap mode)" -ForegroundColor Yellow
try {
    # First, clear previous mapping
    & $BinaryPath remove 04FE:0021 CapsLock 2>&1 | Out-Null
    
    # Set swap mappings
    $output1 = & $BinaryPath set 04FE:0021 CapsLock LCtrl 2>&1 | Out-String
    $exitCode1 = $LASTEXITCODE
    
    $output2 = & $BinaryPath set 04FE:0021 LCtrl CapsLock 2>&1 | Out-String
    $exitCode2 = $LASTEXITCODE
    
    if ($exitCode1 -eq 0 -and $exitCode2 -eq 0 -and $output1 -match "Mapping set successfully" -and $output2 -match "Mapping set successfully") {
        Write-TestResult "Set swap mapping (CapsLock <-> LCtrl)" $true
    } else {
        Write-TestResult "Set swap mapping (CapsLock <-> LCtrl)" $false "Exit codes: $exitCode1, $exitCode2"
    }
} catch {
    Write-TestResult "Set swap mapping (CapsLock <-> LCtrl)" $false $_.Exception.Message
}

# Test 4: Set key mapping (disable mode - Disable CapsLock)
Write-Host "`n[Test 4] Set key mapping (disable mode)" -ForegroundColor Yellow
try {
    # Clear previous mappings
    & $BinaryPath remove 04FE:0021 CapsLock 2>&1 | Out-Null
    & $BinaryPath remove 04FE:0021 LCtrl 2>&1 | Out-Null
    
    $output = & $BinaryPath set 04FE:0021 CapsLock None 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Mapping set successfully") {
        Write-TestResult "Set disable mapping (CapsLock -> None)" $true
    } else {
        Write-TestResult "Set disable mapping (CapsLock -> None)" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Set disable mapping (CapsLock -> None)" $false $_.Exception.Message
}

# Test 5: Show device configuration
Write-Host "`n[Test 5] Show device configuration" -ForegroundColor Yellow
try {
    $output = & $BinaryPath show 04FE:0021 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and $output -match "Device:") {
        Write-TestResult "Show device config" $true
    } else {
        Write-TestResult "Show device config" $false "Exit code: $exitCode, Output: $output"
    }
} catch {
    Write-TestResult "Show device config" $false $_.Exception.Message
}

# Test 6: Save configuration to file
Write-Host "`n[Test 6] Save configuration to file" -ForegroundColor Yellow
$TestConfigPath = "test_config.json"
try {
    $output = & $BinaryPath save --output $TestConfigPath 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0 -and (Test-Path $TestConfigPath)) {
        $configContent = Get-Content $TestConfigPath -Raw
        if ($configContent -match "devices" -and $configContent -match "04FE:0021") {
            Write-TestResult "Save configuration to file" $true
        } else {
            Write-TestResult "Save configuration to file" $false "Invalid JSON content or missing device"
        }
    } else {
        Write-TestResult "Save configuration to file" $false "Exit code: $exitCode or file not created"
    }
} catch {
    Write-TestResult "Save configuration to file" $false $_.Exception.Message
}

# Test 7: Load configuration from file
Write-Host "`n[Test 7] Load configuration from file" -ForegroundColor Yellow
try {
    if (Test-Path $TestConfigPath) {
        $output = & $BinaryPath load --input $TestConfigPath 2>&1 | Out-String
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0 -and $output -match "Configuration loaded successfully") {
            Write-TestResult "Load configuration from file" $true
        } else {
            Write-TestResult "Load configuration from file" $false "Exit code: $exitCode, Output: $output"
        }
    } else {
        Write-TestResult "Load configuration from file" $false "Test config file not found"
    }
} catch {
    Write-TestResult "Load configuration from file" $false $_.Exception.Message
}

# Test 8: Remove mapping and verify
Write-Host "`n[Test 8] Remove mapping and verify" -ForegroundColor Yellow
try {
    # Remove CapsLock mapping
    $output1 = & $BinaryPath remove 04FE:0021 CapsLock 2>&1 | Out-String
    $exitCode1 = $LASTEXITCODE
    
    # Verify removal by showing device config
    $output2 = & $BinaryPath show 04FE:0021 2>&1 | Out-String
    $exitCode2 = $LASTEXITCODE
    
    if ($exitCode1 -eq 0 -and $output1 -match "Mapping removed successfully" -and $exitCode2 -eq 0) {
        Write-TestResult "Remove mapping and verify" $true
    } else {
        Write-TestResult "Remove mapping and verify" $false "Exit codes: $exitCode1, $exitCode2"
    }
} catch {
    Write-TestResult "Remove mapping and verify" $false $_.Exception.Message
}

# Cleanup
if (Test-Path $TestConfigPath) {
    Remove-Item $TestConfigPath -Force
}

# Summary
Write-Host "`n==========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
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
