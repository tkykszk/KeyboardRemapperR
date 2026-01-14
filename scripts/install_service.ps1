# install_service.ps1
# Requires administrator privileges

param(
    [string]$ExePath = ""
)

$serviceName = "KeyboardRemapperR"
$displayName = "Keyboard Remapper R"
$description = "Device-specific keyboard remapper for Windows"

# Check if running as administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "Error: This script must be run as Administrator." -ForegroundColor Red
    Write-Host "Please right-click and select 'Run as Administrator'." -ForegroundColor Yellow
    exit 1
}

# Determine executable path
if ($ExePath -eq "") {
    $ExePath = Join-Path $PSScriptRoot "..\target\release\keyboard-remapper-r.exe"
}

# Check if executable exists
if (-not (Test-Path $ExePath)) {
    Write-Host "Error: Executable not found at: $ExePath" -ForegroundColor Red
    Write-Host "Please build the project first with: cargo build --release" -ForegroundColor Yellow
    exit 1
}

$ExePath = Resolve-Path $ExePath

Write-Host "Installing KeyboardRemapperR service..." -ForegroundColor Cyan
Write-Host "Executable: $ExePath" -ForegroundColor Gray

# Check if service exists
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($service) {
    Write-Host "Service already exists. Stopping and removing..." -ForegroundColor Yellow
    Stop-Service -Name $serviceName -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    sc.exe delete $serviceName
    Start-Sleep -Seconds 2
}

# Create service
Write-Host "Creating service..." -ForegroundColor Cyan
$binPath = "`"$ExePath`" --service"
sc.exe create $serviceName binPath= $binPath start= auto DisplayName= $displayName

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to create service." -ForegroundColor Red
    exit 1
}

# Set description
sc.exe description $serviceName $description

# Start service
Write-Host "Starting service..." -ForegroundColor Cyan
Start-Service -Name $serviceName

if ($?) {
    Write-Host "Service installed and started successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Service Name: $serviceName" -ForegroundColor Gray
    Write-Host "Display Name: $displayName" -ForegroundColor Gray
    Write-Host "Status: Running" -ForegroundColor Gray
    Write-Host ""
    Write-Host "To stop the service, run: Stop-Service -Name $serviceName" -ForegroundColor Yellow
    Write-Host "To uninstall the service, run: .\uninstall_service.ps1" -ForegroundColor Yellow
} else {
    Write-Host "Error: Failed to start service." -ForegroundColor Red
    exit 1
}
