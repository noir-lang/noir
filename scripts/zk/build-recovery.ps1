$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "../..")
$CircuitDir = Join-Path $RepoRoot "zk/noir/recovery"
$ServerArtifactsDir = Join-Path $RepoRoot "demo/server/artifacts"
$ClientPublicDir = Join-Path $RepoRoot "demo/client/public"
$RecoveryArtifact = Join-Path $CircuitDir "target/recovery.json"

Write-Host "--- Noir ZK Build Pipeline ---" -ForegroundColor Cyan

# 1. Compile the circuit
Write-Host "Compiling Noir Recovery Circuit..."
Push-Location $CircuitDir
try {
    if (Get-Command nargo -ErrorAction SilentlyContinue) {
        nargo compile
    } elseif (Test-Path (Join-Path $RepoRoot "target/debug/nargo.exe")) {
        Write-Host "nargo not found in PATH, using local workspace nargo..." -ForegroundColor Yellow
        & (Join-Path $RepoRoot "target/debug/nargo.exe") compile
    } elseif (Test-Path (Join-Path $RepoRoot "target/release/nargo.exe")) {
        Write-Host "nargo not found in PATH, using local workspace nargo..." -ForegroundColor Yellow
        & (Join-Path $RepoRoot "target/release/nargo.exe") compile
    } else {
        throw "nargo not found in PATH or target folder. Please install nargo or compile it first."
    }

    if ($LASTEXITCODE -ne 0) {
        throw "nargo compile failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

if (!(Test-Path $RecoveryArtifact)) {
    throw "Expected recovery artifact was not generated: $RecoveryArtifact"
}

# 2. Sync Artifacts to Server
Write-Host "Syncing artifacts to Demo Server..."
if (!(Test-Path $ServerArtifactsDir)) {
    New-Item -ItemType Directory -Path $ServerArtifactsDir -Force
}
Copy-Item $RecoveryArtifact (Join-Path $ServerArtifactsDir "recovery.json") -Force

# 3. Sync Artifacts to Client (for fetching)
Write-Host "Syncing artifacts to Demo Client..."
if (!(Test-Path $ClientPublicDir)) {
    New-Item -ItemType Directory -Path $ClientPublicDir -Force
}
Copy-Item $RecoveryArtifact (Join-Path $ClientPublicDir "recovery.json") -Force

Write-Host "`nBuild Succeeded! You can now start the server and client." -ForegroundColor Green
Write-Host "Server: cd demo/server; npm install; node index.js"
Write-Host "Client: cd demo/client; npm install; npm run dev"
