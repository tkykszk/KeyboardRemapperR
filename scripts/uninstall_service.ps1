# uninstall_service.ps1
# Requires administrator privileges

$serviceName = "KeyboardRemapperR"

# Check if running as administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "Error: This script must be run as Administrator." -ForegroundColor Red
    Write-Host "Please right-click and select 'Run as Administrator'." -ForegroundColor Yellow
    exit 1
}

Write-Host "Uninstalling KeyboardRemapperR service..." -ForegroundColor Cyan

# Check if service exists
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($service) {
    Write-Host "Stopping service..." -ForegroundColor Yellow
    Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    
    Write-Host "Removing service..." -ForegroundColor Yellow
    sc.exe delete $serviceName
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Service uninstalled successfully!" -ForegroundColor Green
    } else {
        Write-Host "Error: Failed to remove service." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Service not found." -ForegroundColor Yellow
}
