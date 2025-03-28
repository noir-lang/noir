#!/usr/bin/env bash
set -eu

IN=$1
NAME=$(basename $IN .jsonl)
DAT=$(dirname $IN)/$NAME.dat
PNG=$(dirname $IN)/$NAME.png
PLT=$(dirname $0)/bytecode-size-scatter.plt

cat $IN | jq -r '[.name, .base_size, .alt_size, .ratio] | @tsv' > $DAT

gnuplot \
  -e "NAME='$(echo $NAME | tr _ - )'" \
  -e "FILEIN='$DAT'" \
  -e "FILEOUT='$PNG'" \
  $PLT

rm $DAT
