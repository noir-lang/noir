#!/bin/bash
gfind ./src ./test -path ./src/boost -prune -iname *.hpp -o -iname *.cpp -o -iname *.tcc | xargs clang-format -i