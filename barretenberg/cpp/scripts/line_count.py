# Dependency: The line counting utility cloc https://github.com/AlDanial/cloc
import csv
from os import listdir, system
from pathlib import Path

BASE_DIR = Path("src/barretenberg")
PER_FILE_REPORT_PATH = Path("build/per_file_report.csv")

# validate that directory structure hasn't changed since last run
all_dirs = listdir("src/barretenberg")
last_all_dirs = ['bb', 'benchmark', 'commitment_schemes', 'common', 'crypto', 'dsl', 'ecc', 'eccvm', 'env', 'examples', 'flavor', 'goblin', 'grumpkin_srs_gen', 'honk', 'numeric', 'plonk', 'polynomials', 'protogalaxy', 'relations', 'serialize',
                 'smt_verification', 'solidity_helpers', 'srs', 'stdlib', 'sumcheck', 'transcript', 'ultra_honk', 'wasi', 'circuit_checker', 'client_ivc', 'translator_vm', 'vm', 'execution_trace', 'plonk_honk_shared', 'stdlib_circuit_builders', 'proof_system', 'build']
assert (all_dirs == last_all_dirs)

# mark directories that will be covered in an audit of Barretenberg
# calculated the total number of lines weighted by complexity, with maximum complexity equal to 1
dirs_to_audit = [
    'bb',
    # 'benchmark',
    'commitment_schemes',
    # 'common',
    'crypto',
    # 'dsl',
    'ecc',
    'eccvm',
    # 'env',
    # 'examples',
    'flavor',
    'goblin',
    'grumpkin_srs_gen',
    'honk',
    'numeric',
    # 'plonk',
    'polynomials',
    'protogalaxy',
    'relations',
    # 'serialize',
    # 'smt_verification',
    # 'solidity_helpers',
    'srs',
    'stdlib',
    'sumcheck',
    'transcript',
    'ultra_honk',
    # 'wasi',
    'circuit_checker',
    'client_ivc',
    'translator_vm',
    'vm',
    'execution_trace',
    'plonk_honk_shared',
    'stdlib_circuit_builders',
    'proof_system'
]
weights = {directory: 1 for directory in dirs_to_audit}
weights["circuit_checker"] = 0.3
weights["srs"] = 0.3
weights["polynomials"] = 0.5
weights["numeric"] = 0.3
weights["ecc"] = 0.5
weights["crypto"] = 0.5
weights["bb"] = 0.3

TOTAL_NUM_CODE_LINES = 0
# use cloc to count the lines in every file to be audited in the current agreement
system(
    f"cloc --include-lang='C++','C/C++ Header' --by-file --csv --out='{PER_FILE_REPORT_PATH}' {BASE_DIR}")
with open(PER_FILE_REPORT_PATH, newline='') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
        if row['language'] != 'SUM':
            path = Path(row['filename']).relative_to(BASE_DIR).parts[0]
            if path in dirs_to_audit:
                TOTAL_NUM_CODE_LINES += int(row['code']) * weights[path]

TO_AUDIT_NOW = [
    "src/barretenberg/stdlib/primitives/bigfield/bigfield.hpp",
    "src/barretenberg/stdlib/primitives/bigfield/bigfield_impl.hpp",
    "src/barretenberg/stdlib/primitives/bigfield/bigfield.test.cpp",
    "src/barretenberg/stdlib/primitives/field/field.hpp",
    "src/barretenberg/stdlib/primitives/field/field.cpp",
    "src/barretenberg/stdlib/primitives/field/field.test.cpp",
    "src/barretenberg/stdlib/primitives/byte_array/byte_array.hpp",
    "src/barretenberg/stdlib/primitives/byte_array/byte_array.cpp",
    "src/barretenberg/stdlib/primitives/byte_array/byte_array.test.cpp",
    "src/barretenberg/relations/delta_range_constraint_relation.hpp",
    "src/barretenberg/relations/ultra_arithmetic_relation.hpp",
    "src/barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp",
    "src/barretenberg/stdlib_circuit_builders/ultra_circuit_builder.cpp"]
counts = {}
with open(PER_FILE_REPORT_PATH, newline='') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
        if row["filename"] in TO_AUDIT_NOW:
            counts[row["filename"]] = row["code"]
total = 0
for filename, count in counts.items():
    total += int(count)

# print the percentage to be audited
print(f"Audit covers {total/float(TOTAL_NUM_CODE_LINES):.2%}")
