#!/bin/bash
set -e

if [ "$1" == "staged" ]; then
  echo Formatting staged files...
  # we filter barretenberg, see #1149
  for FILE in $(git diff-index --diff-filter=d --relative --cached --name-only HEAD | grep -e '\.\(cpp\|hpp\|tcc\)$' | grep -v barretenberg); do
    clang-format -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
    git add $FILE
  done
elif [ -n "$1" ]; then
  # we filter barretenberg, see #1149
  for FILE in $(git diff-index --relative --name-only $1 | grep -e '\.\(cpp\|hpp\|tcc\)$' | grep -v barretenberg); do
    clang-format -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
  done
else
  # we filter barretenberg, see #1149
  for FILE in $(find ./src -iname *.hpp -o -iname *.cpp -o -iname *.tcc | grep -v src/boost | grep -v barretenberg); do
    clang-format -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
  done
fi
