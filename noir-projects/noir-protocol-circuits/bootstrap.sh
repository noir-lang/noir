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

yarn
node ./generate_variants.js

echo "Compiling protocol circuits..."
NARGO=${NARGO:-../../noir/noir-repo/target/release/nargo}
$NARGO compile --silence-warnings

BB_HASH=${BB_HASH:-$(cd ../../ && git ls-tree -r HEAD | grep 'barretenberg/cpp' | awk '{print $3}' | git hash-object --stdin)}
echo Using BB hash $BB_HASH
mkdir -p "./target/keys"

AVAILABLE_MEMORY=0

case "$(uname)" in
  Linux*)
    # Check available memory on Linux
    AVAILABLE_MEMORY=$(awk '/MemTotal/ { printf $2 }' /proc/meminfo)
    ;;
  *)
    echo "Parallel vk generation not supported on this operating system"
    ;;
esac
# This value may be too low.
# If vk generation fail with an amount of free memory greater than this value then it should be increased.
MIN_PARALLEL_VK_GENERATION_MEMORY=500000000

if [[ AVAILABLE_MEMORY -lt MIN_PARALLEL_VK_GENERATION_MEMORY ]]; then
  echo "System does not have enough memory for parallel vk generation, falling back to sequential"

  for pathname in "./target"/*.json; do    
      BB_HASH=$BB_HASH node ./scripts/generate_vk_json.js "$pathname" "./target/keys"
  done

else

  echo "Generating vks in parallel..."
  for pathname in "./target"/*.json; do    
      BB_HASH=$BB_HASH node ./scripts/generate_vk_json.js "$pathname" "./target/keys" &
  done

  for job in $(jobs -p); do
    wait $job || exit 1
  done

fi

