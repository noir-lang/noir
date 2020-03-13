#!/bin/bash
if [ "$1" == "staged" ]; then
  for FILE in $(git diff-index --relative --cached --name-only HEAD | grep -e '\.\(cpp\|hpp\|tcc\)$'); do
    clang-format -i $FILE
    git add $FILE
  done
else
  gfind ./src -iname *.hpp -o -iname *.cpp -o -iname *.tcc | grep -v src/boost | xargs clang-format -i
fi