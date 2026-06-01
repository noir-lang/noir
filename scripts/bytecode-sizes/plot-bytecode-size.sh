#!/usr/bin/env bash
set -eu

IN=$1
NAME=$(basename $IN .jsonl)
DAT=$(dirname $IN)/$NAME.dat
PLT=$(dirname $0)/bytecode-size-scatter.plt

# TSV columns:
#   1: name
#   2: compressed_base   3: compressed_alt   4: compressed_ratio
#   5: uncompressed_base 6: uncompressed_alt 7: uncompressed_ratio
cat $IN | jq -r '[
    .name,
    .compressed_base, .compressed_alt, .compressed_ratio,
    .uncompressed_base, .uncompressed_alt, .uncompressed_ratio
] | @tsv' > $DAT

# Plot compressed (the wire size that ships) and uncompressed (pre-gzip
# raw format size) separately. The two ratios can diverge because gzip
# rewards redundancy that a denser pre-gzip encoding has already removed.
for kind in compressed uncompressed; do
    case $kind in
        compressed)   x_col=2; ratio_col=4 ;;
        uncompressed) x_col=5; ratio_col=7 ;;
    esac
    PNG=$(dirname $IN)/$NAME-$kind.png
    gnuplot \
        -e "NAME='$(echo $NAME | tr _ - ) ($kind)'" \
        -e "FILEIN='$DAT'" \
        -e "FILEOUT='$PNG'" \
        -e "X_COL=$x_col" \
        -e "RATIO_COL=$ratio_col" \
        $PLT
done

rm $DAT
