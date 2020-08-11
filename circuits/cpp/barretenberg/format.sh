#!/bin/bash
if [ "$1" == "staged" ]; then
  echo Formatting staged files...
  for FILE in $(git diff-index --diff-filter=d --relative --cached --name-only HEAD | grep -e '\.\(cpp\|hpp\|tcc\)$'); do
    clang-format -i $FILE
    git add $FILE
  done
elif [ -n "$1" ]; then
  git diff-index --relative --name-only $1 | grep -e '\.\(cpp\|hpp\|tcc\)$' | xargs clang-format --sort-includes -i
else
  gfind ./src -iname *.hpp -o -iname *.cpp -o -iname *.tcc | grep -v src/boost | xargs clang-format --sort-includes -i
fi