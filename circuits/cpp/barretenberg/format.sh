#!/bin/bash
if [ "$1" == "staged" ]; then
  for FILE in $(git diff-index --relative --cached --name-only HEAD | grep -e '\.\(cpp\|hpp\|tcc\)$'); do
    clang-format -i $FILE
    git add $FILE
  done
else
  gfind ./src ./test -path ./src/boost -prune -iname *.hpp -o -iname *.cpp -o -iname *.tcc | xargs clang-format -i
fi