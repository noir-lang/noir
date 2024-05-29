#!/usr/bin/env python3
import json
import sys

TIME_COUNTERS_USED = ["commit(t)", "Goblin::merge(t)"]

def modify_benchmark_data(file_paths, prefixes):
    combined_results = {"benchmarks": []}

    for file_path, prefix in zip(file_paths, prefixes):
        with open(file_path, 'r') as file:
            data = json.load(file)
            # Modify benchmark names and extract specific data
            for benchmark in data['benchmarks']:
                # Prefix benchmark names
                benchmark['name'] = f"{prefix}{benchmark['name']}"
                benchmark['run_name'] = f"{prefix}{benchmark['run_name']}"

                if prefix != "":
                    combined_results['benchmarks'].append(benchmark)
                # Isolate batch_mul_with_endomorphism
                for counter in TIME_COUNTERS_USED:
                    if counter in benchmark:
                        new_entry = {
                            "name": f"{counter}",
                            "run_name": benchmark['run_name'],
                            "run_type": benchmark['run_type'],
                            "repetitions": benchmark['repetitions'],
                            "repetition_index": benchmark['repetition_index'],
                            "threads": benchmark['threads'],
                            "iterations": benchmark['iterations'],
                            "real_time": benchmark[counter],
                            "cpu_time": benchmark[counter],
                            "time_unit": "ns"
                        }
                        combined_results['benchmarks'].append(new_entry)

    return combined_results

# Using command line arguments to get prefixes and file paths
if len(sys.argv) < 3 or len(sys.argv) % 2 != 1:
    print("Usage: python script.py <Prefix1> <file1> <Prefix2> <file2> ...")
    sys.exit(1)

prefixes = sys.argv[1::2]
file_paths = sys.argv[2::2]

final_data = modify_benchmark_data(file_paths, prefixes)

# Save the combined results to a file
print(json.dumps(final_data, indent=4))

