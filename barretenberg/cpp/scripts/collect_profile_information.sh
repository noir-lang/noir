#!/bin/bash
set -eu

PRESET=${1:-xray-1thread} # can also be 'xray'
ONLY_PROCESS=${2:-}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build with xray preset.
cmake --preset $PRESET
cmake --build --preset $PRESET

cd build-$PRESET

if [ -z "$ONLY_PROCESS" ]; then
  # Clear old profile data.
  rm -f xray-log.honk_bench_main_simple.*

  # Run benchmark with profiling.
  XRAY_OPTIONS="patch_premain=true xray_mode=xray-basic verbosity=1" ./bin/honk_bench_main_simple
fi

function shorten_cpp_names() {
  NO_TEMP='s/<[^<>;]+>//g;'
  sed -E '# Multiple rounds of template removal (crude but simple).
          '"$NO_TEMP $NO_TEMP $NO_TEMP $NO_TEMP $NO_TEMP $NO_TEMP"'
          # Remove problematic trailing const.
          s/ const;/;/g;
          # Parameter removal.
          s/\([^();]*\)/()/g;
          # Return value removal.
          s/;[^; ]+ /;/g;
          # Remove namespaces.
          s/[a-zA-Z_][a-zA-Z0-9_]*:://g;
         '
}

# Process benchmark file.
llvm-xray-16 stack xray-log.honk_bench_main_simple.* \
  --instr_map=./bin/honk_bench_main_simple --stack-format=flame --aggregate-threads --aggregation-type=time --all-stacks \
  | node ../scripts/llvm_xray_stack_flame_corrector.js \
  | shorten_cpp_names \
  | ../scripts/flamegraph.pl --width 1200 --fontsize 10 \
  > xray.svg
echo "Profiling complete, now you can do e.g. 'scp mainframe:`readlink -f xray.svg` .' on a local terminal and open the SVG in a browser."
