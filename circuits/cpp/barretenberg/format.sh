#!/bin/bash
set -e

if [ "$1" == "staged" ]; then
  echo Formatting staged files...
  for FILE in $(git diff-index --diff-filter=d --relative --cached --name-only HEAD | grep -e '\.\(cpp\|hpp\|tcc\)$'); do
    clang-format -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
    git add $FILE
  done
elif [ -n "$1" ]; then
  for FILE in $(git diff-index --relative --name-only $1 | grep -e '\.\(cpp\|hpp\|tcc\)$'); do
    clang-format --sort-includes -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
  done
else
  for FILE in $(find ./src -iname *.hpp -o -iname *.cpp -o -iname *.tcc | grep -v src/boost); do
    clang-format --sort-includes -i $FILE
    sed -i.bak 's/\r$//' $FILE && rm ${FILE}.bak
  done
fi
