$ErrorActionPreference = "Stop"

$CircuitDir = "zk/noir/recovery"
$ServerArtifactsDir = "demo/server/artifacts"
$ClientPublicDir = "demo/client/public"

Write-Host "--- Noir ZK Build Pipeline ---" -ForegroundColor Cyan

# 1. Compile the circuit
Write-Host "Compiling Noir Recovery Circuit..."
Push-Location $CircuitDir
nargo compile
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
