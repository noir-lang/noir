import os
import shutil
import subprocess
import sys
from pathlib import Path
from concurrent.futures import ProcessPoolExecutor, as_completed

# Reactive `regression_7323` once enums are ready
EXCLUDED_DIRS = {
    "workspace",
    "workspace_default_member",
    "regression_7323"
}

def process_dir(dir_path, current_dir):
    dir_name = dir_path.name
    log_file = current_dir / "rebuild.log"
    
    try:
        if not (dir_path / "Nargo.toml").exists():
            return f"{dir_path}: skipped (no Nargo.toml)"

        artifacts_dir = current_dir / "acir_artifacts" / dir_name
        artifacts_target = artifacts_dir / "target"
        artifacts_proofs = artifacts_dir / "proofs"
        
        artifacts_target.mkdir(parents=True, exist_ok=True)
        artifacts_proofs.mkdir(parents=True, exist_ok=True)

        # Clean target directory in test program
        target_dir = dir_path / "target"
        if target_dir.exists():
            shutil.rmtree(target_dir)

        # Run nargo execute witness
        result = subprocess.run(
            ["nargo", "execute", "witness"],
            cwd=dir_path,
            capture_output=True,
            text=True
        )

        with open(log_file, "a", encoding="utf-8") as f:
            f.write(f"Processing {dir_path}\n")
            f.write(result.stdout)
            f.write(result.stderr)

        if result.returncode != 0:
            with open(log_file, "a", encoding="utf-8") as f:
                f.write(f"{dir_path} failed\n")
            return f"{dir_path} failed"

        # Move artifacts
        if artifacts_target.exists():
            shutil.rmtree(artifacts_target)
        artifacts_target.mkdir()

        program_json = target_dir / f"{dir_name}.json"
        if program_json.exists():
            shutil.move(program_json, artifacts_target / "program.json")
        
        for gz_file in target_dir.glob("*.gz"):
            shutil.move(gz_file, artifacts_target / gz_file.name)

        with open(log_file, "a", encoding="utf-8") as f:
            f.write(f"{dir_path} succeeded\n")
        return f"{dir_path} succeeded"

    except Exception as e:
        error_msg = f"{dir_path} exception: {str(e)}\n"
        with open(log_file, "a", encoding="utf-8") as f:
            f.write(error_msg)
        return f"{dir_path} failed"

def main():
    current_dir = Path.cwd()
    base_path = current_dir / "execution_success"
    benchmarks_path = current_dir / "benchmarks"
    
    # Remove existing artifacts
    artifacts_root = current_dir / "acir_artifacts"
    if artifacts_root.exists():
        shutil.rmtree(artifacts_root)
    artifacts_root.mkdir()

    # Gather directories to process
    dirs_to_process = []
    if len(sys.argv) > 1:
        for arg in sys.argv[1:]:
            dirs_to_process.append(base_path / arg)
    else:
        if base_path.exists():
            for d in base_path.iterdir():
                if d.is_dir() and d.name not in EXCLUDED_DIRS:
                    dirs_to_process.append(d)
        
        if benchmarks_path.exists():
            for d in benchmarks_path.iterdir():
                if d.is_dir() and d.name not in EXCLUDED_DIRS:
                    dirs_to_process.append(d)

    log_file = current_dir / "rebuild.log"
    if log_file.exists():
        log_file.unlink()

    failed_dirs = []
    
    # Process in parallel
    with ProcessPoolExecutor(max_workers=os.cpu_count()) as executor:
        futures = {executor.submit(process_dir, d, current_dir): d for d in dirs_to_process}
        for future in as_completed(futures):
            res = future.result()
            print(res)
            if "failed" in res:
                failed_dirs.append(res.split(":")[0])

    if failed_dirs:
        print("\nRebuild failed for the following directories:")
        for d in failed_dirs:
            print(f"- {d}")
        sys.exit(1)
    else:
        print("\nRebuild Succeeded!")

if __name__ == "__main__":
    main()
