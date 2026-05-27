import os
import json
import subprocess
import sys
from pathlib import Path

# These tests are incompatible with gas reporting
EXCLUDED_DIRS = {
    "workspace",
    "workspace_default_member",
    "databus",
    "databus_composite_calldata",
    "databus_two_calldata",
    "databus_two_calldata_simple",
    "fold_2_to_17",
    "fold_after_inlined_calls",
    "fold_basic",
    "fold_basic_nested_call",
    "fold_call_witness_condition",
    "fold_complex_outputs",
    "fold_distinct_return",
    "fold_fibonacci",
    "fold_numeric_generic_poseidon",
    "regression_7143",
    "regression_7612"
}

def main():
    backend = os.environ.get("BACKEND", "bb")
    current_dir = Path.cwd()
    artifacts_path = current_dir / "acir_artifacts"
    
    if not artifacts_path.exists():
        print(f"Error: {artifacts_path} does not exist. Run rebuild.py first.")
        sys.exit(1)

    programs = []
    
    artifact_dirs = [d for d in artifacts_path.iterdir() if d.is_dir()]
    
    for artifact_dir in artifact_dirs:
        artifact_name = artifact_dir.name
        if artifact_name in EXCLUDED_DIRS:
            continue
            
        print(f"Processing {artifact_name}")
        
        program_json = artifact_dir / "target" / "program.json"
        if not program_json.exists():
            print(f"Warning: {program_json} not found. Skipping.")
            continue
            
        try:
            result = subprocess.run(
                [backend, "gates", "-b", str(program_json)],
                capture_output=True,
                text=True,
                check=True
            )
            
            gates_info = json.loads(result.stdout)
            
            if "functions" in gates_info and len(gates_info["functions"]) > 0:
                main_fn = gates_info["functions"][0]
                program_info = {
                    "package_name": artifact_name,
                    "functions": [{
                        "name": "main",
                        "acir_opcodes": main_fn.get("acir_opcodes"),
                        "opcodes": main_fn.get("acir_opcodes"),
                        "circuit_size": main_fn.get("circuit_size")
                    }],
                    "unconstrained_functions": []
                }
                programs.append(program_info)
            else:
                print(f"Warning: No functions found in gates info for {artifact_name}")
                
        except subprocess.CalledProcessError as e:
            print(f"Error running {backend} gates for {artifact_name}: {e.stderr}")
        except json.JSONDecodeError as e:
            print(f"Error parsing JSON for {artifact_name}: {str(e)}")
        except Exception as e:
            print(f"Unexpected error for {artifact_name}: {str(e)}")

    report = {"programs": programs}
    
    with open("gates_report.json", "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
        
    print("\nGates report generated in gates_report.json")

if __name__ == "__main__":
    main()
