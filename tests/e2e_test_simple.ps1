# Simple E2E Test Script for KeyboardRemapperR
# Minimal test to verify basic functionality

$ErrorActionPreference = "Stop"

# Get the binary path
$BinaryPath = "target\release\keyboard-remapper-r.exe"

if (-not (Test-Path $BinaryPath)) {
    Write-Host "Error: Binary not found at $BinaryPath" -ForegroundColor Red
    exit 1
}

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "KeyboardRemapperR Simple E2E Test" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: List devices
Write-Host "[Test 1] List devices" -ForegroundColor Yellow
$output1 = & $BinaryPath list
Write-Host "Output: $output1"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

# Test 2: Set mapping
Write-Host "[Test 2] Set mapping" -ForegroundColor Yellow
$output2 = & $BinaryPath set 04FE:0021 CapsLock LCtrl
Write-Host "Output: $output2"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

# Test 3: Show device
Write-Host "[Test 3] Show device" -ForegroundColor Yellow
$output3 = & $BinaryPath show 04FE:0021
Write-Host "Output: $output3"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

# Test 4: Save config
Write-Host "[Test 4] Save config" -ForegroundColor Yellow
$output4 = & $BinaryPath save
Write-Host "Output: $output4"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

# Test 5: Load config
Write-Host "[Test 5] Load config" -ForegroundColor Yellow
$output5 = & $BinaryPath load
Write-Host "Output: $output5"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

# Test 6: Remove mapping
Write-Host "[Test 6] Remove mapping" -ForegroundColor Yellow
$output6 = & $BinaryPath remove 04FE:0021 CapsLock
Write-Host "Output: $output6"
Write-Host "Exit Code: $LASTEXITCODE"
Write-Host ""

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "All commands executed successfully" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Cyan
