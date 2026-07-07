#!/usr/bin/env bash
set -eu

IN1=$1
IN2=$2

# Join records from the two input files by program name and emit two ratios:
# `compressed_ratio` and `uncompressed_ratio`. The latter isolates the
# wire-format cost from gzip's contribution — handy for diagnosing cases
# where the compressed ratio regresses despite the format being denser.
jq --slurp -c '
. as $top |
($top[] | select(.encoding == "base") | .data[]) as $base |
($top[] | select(.encoding == "alt")  | .data[] | select(.name == $base.name)) as $alt |
{
    name: $base.name,
    compressed_base: $base.compressed_size,
    compressed_alt: $alt.compressed_size,
    compressed_ratio: ($alt.compressed_size / $base.compressed_size),
    uncompressed_base: $base.uncompressed_size,
    uncompressed_alt: $alt.uncompressed_size,
    uncompressed_ratio: ($alt.uncompressed_size / $base.uncompressed_size)
}
' \
    <(cat $IN1 | jq --slurp '{encoding: "base", data: .}') \
    <(cat $IN2 | jq --slurp '{encoding: "alt", data: .}')
