#!/usr/bin/env bash
set -eu

TMP=tmp.dot
RESULT_DOT=barretenberg_modules.dot
RESULT_PNG=barretenberg_modules.png

# initialize a directed graph for graphviz
echo digraph BarretenbergModules { > $TMP
# populate the directed graph
for file in $(find ./src/barretenberg/ -iname CMakeLists.txt); do
  opening_chars=$(head -c 19 "$file")
    if [ "$opening_chars" == barretenberg_module ]; then
      awk -f ./scripts/barretenberg_module_digraph_edges.awk $file >> $TMP
    fi
done
echo } >> $TMP

# apply transitive reduction to remove dependcies that are implied by other dependencies
cat $TMP | tred > $RESULT_DOT
rm $TMP

# produce a PNG of the graph
dot -Tpng $RESULT_DOT -o $RESULT_PNG