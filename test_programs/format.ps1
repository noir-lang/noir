$Nargo = if ($env:NARGO) { $env:NARGO } else { "nargo" }

# These tests are incompatible with gas reporting
$ExcludedDirs = @("workspace", "overlapping_dep_and_mod", "overlapping_dep_and_mod_fix", "workspace_default_member", "workspace_reexport_bug")

$CurrentDir = Get-Location

function Collect-Dirs($Path) {
    $TestDirs = Get-ChildItem -Path "$CurrentDir/$Path" -Directory
    $Results = @()

    foreach ($Dir in $TestDirs) {
        $DirName = $Dir.Name
        # skip generated tests
        if ($DirName -match "^(noirc_frontend_|noirc_evaluator_)") {
            continue
        }

        if ($ExcludedDirs -contains $DirName) {
            continue
        }

        if (-not (Test-Path "$CurrentDir/$Path/$DirName/Nargo.toml")) {
            Write-Host "No Nargo.toml found in $($DirName). Removing directory."
            Remove-Item -Path "$CurrentDir/$Path/$DirName" -Recurse -Force
            Write-Host "$($DirName): skipped (no Nargo.toml)"
            continue
        }

        $Results += "  `"$Path/$DirName`","
    }
    return $Results
}

$NargoTomlContent = @("[workspace]", "members = [")

$NargoTomlContent += Collect-Dirs "compile_success_empty"
$NargoTomlContent += Collect-Dirs "compile_success_contract"
$NargoTomlContent += Collect-Dirs "compile_success_no_bug"
$NargoTomlContent += Collect-Dirs "compile_success_with_bug"
$NargoTomlContent += Collect-Dirs "execution_success"
$NargoTomlContent += Collect-Dirs "noir_test_success"
$NargoTomlContent += Collect-Dirs "fuzzing_failure"

$NargoTomlContent += "]"

$NargoTomlPath = "$CurrentDir/Nargo.toml"
$NargoTomlContent | Out-File -FilePath $NargoTomlPath -Encoding utf8

try {
    if ($args[0] -eq "check") {
        & $Nargo fmt --check
    } else {
        & $Nargo fmt
    }
} finally {
    if (Test-Path $NargoTomlPath) {
        Remove-Item $NargoTomlPath
    }
}
