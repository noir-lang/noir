$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "../..")
$ClientDir = Join-Path $RepoRoot "demo/client"

Write-Host "--- Noir ZK Build Pipeline ---" -ForegroundColor Cyan
Write-Host "Generating Noir Recovery Circuit artifact with @noir-lang/noir_wasm..."

Push-Location $ClientDir
try {
    npm run build:recovery-artifact
    if ($LASTEXITCODE -ne 0) {
        throw "artifact generation failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "`nBuild Succeeded! You can now restart the server and client." -ForegroundColor Green
Write-Host "Server: cd demo/server; node index.js"
Write-Host "Client: cd demo/client; npm run dev"
