#!/usr/bin/env bash
set -eu

IN1=$1
IN2=$2

jq --slurp -c '
. as $top |
($top[] | select(.encoding == "base") | .data[]) as $base |
($top[] | select(.encoding == "alt")  | .data[] | select(.name == $base.name)) as $alt |
{
    name: $base.name,
    base_size: $base.bytecode_size,
    alt_size: $alt.bytecode_size,
    ratio: ($alt.bytecode_size / $base.bytecode_size)
}
' \
    <(cat $IN1 | jq --slurp '{encoding: "base", data: .}') \
    <(cat $IN2 | jq --slurp '{encoding: "alt", data: .}') \
