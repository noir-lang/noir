#!/usr/bin/python3
"""
Tool for analysing several benchmarks from basics_bench to calculate operation timings
For example, in src directory:
python3 ../src/barretenberg/benchmark/basics_bench/analyse_all_benchmarks.py -f bin/basics_bench
"""
import argparse
import subprocess
import tempfile
from single_benchmark_analysis import evaluate_benchmark_from_file
import os

# Some of the benchmarks use other operations to randomise the procedure, so we need to subtract the results
filter_rules={
    "sequential_copy":"cycle_waste",
    "cycle_waste":None,
    "parallel_for_field_element_addition:":None,
    "ff_addition":"cycle_waste",
    "ff_multiplication":"cycle_waste",
    "ff_sqr":"cycle_waste",
    "ff_invert":"ff_addition",
    "ff_to_montgomery":"cycle_waste",
    "ff_from_montgomery":"cycle_waste",
    "ff_reduce":"ff_addition",
    "projective_point_addition":"cycle_waste",
    "projective_point_accidental_doubling":"cycle_waste",
    "projective_point_doubling":"cycle_waste",
    "scalar_multiplication":"ff_addition",
}
def get_benchmarks(filename):
    """
    Get a list of benchmarks from the binary
    """
    result=subprocess.run([filename,"--benchmark_list_tests"],capture_output=True)
    result.check_returncode()
    output_lines=result.stdout.splitlines()
    benchmark_names=set([x.decode().split('/')[0] for x in output_lines])
    return sorted(list(benchmark_names))

def run_benchmarks(filename,bnames):
    """
    Run benchmarks for each type and collect results
    """
    benchmark_results=dict()
    for bname in bnames:
        output_file=tempfile.mktemp()
        result=subprocess.run([filename,f"--benchmark_filter={bname}.*",f"--benchmark_out={output_file}","--benchmark_out_format=csv"])
        result.check_returncode()
        benchmark_result=evaluate_benchmark_from_file(output_file)*1000
        benchmark_results[bname]=benchmark_result
        print (f"Benchmark {bname} unfiltered: {benchmark_result} ns")
        os.remove(output_file)

    return benchmark_results

def filter_benchmarks(benchmark_results):
    """
    Apply filtering rules and print the benchmarks
    """
    global filter_rules
    print ("Filtered benchmark results:")
    max_len=0
    for bname in sorted(benchmark_results.keys()):
        if len(bname)>max_len:
            max_len=len(bname)
    for bname in sorted(benchmark_results.keys()):
        if bname not in filter_rules.keys() or filter_rules[bname]==None:
            print(f"\t{bname}:{' '*(max_len-len(bname))}\t{benchmark_results[bname]:.1f}")
        else:
            print(f"\t{bname}:{' '*(max_len-len(bname))}\t{benchmark_results[bname]-benchmark_results[filter_rules[bname]]:.1f}")

if __name__=="__main__":
    parser=argparse.ArgumentParser(description='Run all the individual benchmarks',epilog='This expects a single file with a single type of benchmark <name>/i')
    parser.add_argument("-f","--file",dest="filename",required=True,help="run benchmark FILE", metavar="FILE")
    args=parser.parse_args()
    filename=args.filename
    if filename==None:
        parser.print_help()
        exit()
    benchmark_names=get_benchmarks(filename)
    print("Will run the following benchmarks:")
    for bname in benchmark_names:
        print(f'\t{bname}')
    unfiltered_results=run_benchmarks(filename,benchmark_names)
    filter_benchmarks(unfiltered_results)
    

    