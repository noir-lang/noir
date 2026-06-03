import os
import sys
import subprocess
import shutil
import platform
import json
from pathlib import Path

# Tests to be profiled for memory report
TESTS_TO_PROFILE = [
    "execution_success/workspace",
    "execution_success/regression_4709",
    "compile_success_no_bug/ram_blowup_regression",
    "execution_success/global_var_regression_entry_points"
]

def parse_memory(mem_string):
    mem_string = mem_string.replace(",", "").strip()
    if "GB" in mem_string:
        num = float(mem_string.replace("GB", "").strip())
        return num * 1024
    elif "MB" in mem_string:
        num = float(mem_string.replace("MB", "").strip())
        return num
    elif "KB" in mem_string:
        num = float(mem_string.replace("KB", "").strip())
        return num / 1024
    elif "B" in mem_string:
        num = float(mem_string.replace("B", "").strip())
        return num / (1024 * 1024)
    else:
        try:
            return float(mem_string) / (1024 * 1024)
        except:
            return 0.0

def main():
    if platform.system() != "Linux":
        print(f"Memory report currently requires heaptrack, which is only supported on Linux. Skipping on {platform.system()}.")
        # Write an empty report to satisfy potential CI steps
        with open("memory_report.json", "w") as f:
            f.write("[]")
        return

    nargo = os.environ.get("NARGO", "nargo")
    current_dir = Path.cwd()
    base_path = current_dir

    profile_list = TESTS_TO_PROFILE
    if len(sys.argv) > 1 and sys.argv[1] == "1":
        profile_list = ["."]

    flags = os.environ.get("FLAGS", "")
    
    report = []

    for test_path_str in profile_list:
        test_path = base_path / test_path_str
        os.chdir(test_path)

        test_name = test_path.name if sys.argv[1] == "1" else Path(test_path_str).name
        
        if not test_name:
            print("test name is empty")
            sys.exit(1)

        command = f"compile --force --silence-warnings {flags}".split()
        if len(sys.argv) > 2 and sys.argv[2] == "1":
            command = "execute --silence-warnings".split()

        heap_output = current_dir / f"{test_name}_heap"
        
        subprocess.run(["heaptrack", "--output", str(heap_output), nargo] + command)
        
        heap_file = None
        if (current_dir / f"{test_name}_heap.gz").exists():
            heap_file = current_dir / f"{test_name}_heap.gz"
        elif (current_dir / f"{test_name}_heap.zst").exists():
            heap_file = current_dir / f"{test_name}_heap.zst"
            
        if heap_file:
            analysis_file = current_dir / f"{test_name}_heap_analysis.txt"
            with open(analysis_file, "w") as f:
                subprocess.run(["heaptrack", "--analyze", str(heap_file)], stdout=f)
            heap_file.unlink()
            
            peak_memory_val = 0.0
            with open(analysis_file, "r") as f:
                for line in f:
                    if "peak heap memory consumption" in line:
                        # peak heap memory consumption: 123.45 MB (peak simulation memory consumption: ...)
                        consumption_str = line.split(":")[1].split("(")[0].strip()
                        peak_memory_val = parse_memory(consumption_str)
                        break
            
            analysis_file.unlink()
            report.append({
                "name": test_name,
                "value": peak_memory_val,
                "unit": "MB"
            })
        
        os.chdir(current_dir)

    with open("memory_report.json", "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    print("\nMemory report generated in memory_report.json")

if __name__ == "__main__":
    main()
