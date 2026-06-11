#!/usr/bin/env bash
set -eu

AZTEC_PACKAGES_DIR=$1

# The `bytecode` field in nargo's compiled artifact is `base64(gzip(format_byte
# + msgpack_bytes))`. We emit both:
#
# * `compressed_size`   — length of the base64-decoded blob (the gzipped
#                         payload that ships on the wire). Matches what the
#                         prior version of this script reported.
# * `uncompressed_size` — length after un-gzipping. The "raw" size, useful
#                         for separating wire-format cost from gzip's
#                         compression ratio.
#
# Writes a base64-decoded copy of the bytecode to a temp file once per
# program so we can `wc -c` it directly and pipe it through `gzip -d`
# without re-decoding.
emit_sizes() {
    local name="$1"
    local b64="$2"
    local tmp
    tmp=$(mktemp)
    printf '%s' "$b64" | base64 -d > "$tmp"
    local compressed uncompressed
    compressed=$(wc -c < "$tmp" | tr -d '[:space:]')
    uncompressed=$(gzip -dc "$tmp" | wc -c | tr -d '[:space:]')
    rm -f "$tmp"
    jq -nc \
        --arg name "$name" \
        --argjson compressed_size "$compressed" \
        --argjson uncompressed_size "$uncompressed" \
        '{name: $name, compressed_size: $compressed_size, uncompressed_size: $uncompressed_size}'
}

for file in "$AZTEC_PACKAGES_DIR/noir-projects/noir-protocol-circuits/target/"*.json; do
    program=$(basename "$file" .json)
    b64=$(jq -r '.bytecode' "$file")
    emit_sizes "$program" "$b64"
done

for file in "$AZTEC_PACKAGES_DIR/noir-projects/noir-contracts/target/"*.json; do
    contract=$(basename "$file" .json)
    # Each contract file has multiple functions, each with its own bytecode.
    # `@tsv` puts the function name and its base64 bytecode on one line each.
    while IFS=$'\t' read -r fname b64; do
        emit_sizes "${contract}::${fname}" "$b64"
    done < <(jq -r '.functions | sort_by(.name) | .[] | [.name, .bytecode] | @tsv' "$file")
done
