#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

CMD=${1:-}

if [ -n "$CMD" ]; then
  if [ "$CMD" = "clean" ]; then
    git clean -fdx
    exit 0
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

# Attempt to just pull artefacts from CI and exit on success.
[ -n "${USE_CACHE:-}" ] && ./bootstrap_cache.sh && exit

g="\033[32m"  # Green
b="\033[34m"  # Blue
r="\033[0m"   # Reset

((cd "./noir-contracts" && ./bootstrap.sh) > >(awk -v g="$g" -v r="$r" '{print g "contracts: " r $0}')) &
((cd "./noir-protocol-circuits" && ./bootstrap.sh) > >(awk -v b="$b" -v r="$r" '{print  b "protocol-circuits: " r $0}')) &

for job in $(jobs -p); do
  wait $job || exit 1
done
