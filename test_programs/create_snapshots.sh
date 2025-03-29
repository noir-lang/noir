#!/usr/bin/env bash
set -e


SSA=$(nargo --program-dir $1 compile --show-ssa)

tmp=$(mktemp -d -p .)
trap "rm -rf $tmp" EXIT

csplit -z --prefix=$tmp/ - "/fn main/" "{*}" <<<"$SSA" >> /dev/null

OUT_DIR=../compiler/noirc_evaluator/src/ssa/opt/snapshots/$1
mkdir -p $OUT_DIR

NUM=0
LAST="NONE"
for file in $(ls $tmp); do
    firstline=$(sed 's/://g;s/`//g;s/After //g;s/(1st)/1/g;s/(2nd)/2/g;s/(3rd)/3/g;s/(4th)/4/g;s/ /_/g;' <<<$(tail -1 $tmp/$file))
    CURRENT=$(tr '[:upper:]' '[:lower:]' <<<$firstline)

    sed '$d' $tmp/$file | sed -e :a -e '/^\n*$/{$d;N;};/\n$/ba'  > $OUT_DIR/$(printf '%02d' $NUM)_$CURRENT
    LAST=$CURRENT
    NUM=$(($NUM + 1))
done


