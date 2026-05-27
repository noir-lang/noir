$ErrorActionPreference = "Stop"

$CircuitDir = "zk/noir/recovery"
$ServerArtifactsDir = "demo/server/artifacts"
$ClientPublicDir = "demo/client/public"

Write-Host "--- Noir ZK Build Pipeline ---" -ForegroundColor Cyan

# 1. Compile the circuit
Write-Host "Compiling Noir Recovery Circuit..."
Push-Location $CircuitDir
if (Get-Command nargo -ErrorAction SilentlyContinue) {
    nargo compile
} elseif (Test-Path "$PSScriptRoot/../../target/debug/nargo.exe") {
    Write-Host "nargo not found in PATH, using local workspace nargo..." -ForegroundColor Yellow
    & "$PSScriptRoot/../../target/debug/nargo.exe" compile
} elseif (Test-Path "$PSScriptRoot/../../target/release/nargo.exe") {
    Write-Host "nargo not found in PATH, using local workspace nargo..." -ForegroundColor Yellow
    & "$PSScriptRoot/../../target/release/nargo.exe" compile
} else {
    Write-Host "nargo not found in PATH or target folder. Please install nargo or compile it first." -ForegroundColor Red
    exit 1
}
Pop-Location

# 2. Sync Artifacts to Server
Write-Host "Syncing artifacts to Demo Server..."
if (!(Test-Path $ServerArtifactsDir)) {
    New-Item -ItemType Directory -Path $ServerArtifactsDir -Force
}
Copy-Item "$CircuitDir/target/recovery.json" "$ServerArtifactsDir/recovery.json" -Force

# 3. Sync Artifacts to Client (for fetching)
Write-Host "Syncing artifacts to Demo Client..."
if (!(Test-Path $ClientPublicDir)) {
    New-Item -ItemType Directory -Path $ClientPublicDir -Force
}
Copy-Item "$CircuitDir/target/recovery.json" "$ClientPublicDir/recovery.json" -Force

Write-Host "`nBuild Succeeded! You can now start the server and client." -ForegroundColor Green
Write-Host "Server: cd demo/server; npm install; node index.js"
Write-Host "Client: cd demo/client; npm install; npm run dev"
