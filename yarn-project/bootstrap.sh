#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

# Check node version.
node_version=$(node -v | tr -d 'v')
major=${node_version%%.*}
rest=${node_version#*.}
minor=${rest%%.*}

YELLOW="\033[93m"
BLUE="\033[34m"
GREEN="\033[32m"
BOLD="\033[1m"
RESET="\033[0m"

if ((major < 18 || (major == 18 && minor < 19))); then
  echo "Node.js version is less than 18.19. Exiting."
  exit 1
fi

cd "$(dirname "$0")"

CMD=${1:-}

if [ "$CMD" = "clean" ]; then
  git clean -fdx
  exit 0
elif [ "$CMD" = "full" ]; then
  yarn install --immutable
  yarn build
  exit 0
elif [[ -n "$CMD" && "$CMD" != "fast" ]]; then
  echo "Unknown command: $CMD"
  exit 1
fi

# Fast build does not delete everything first.
# It regenerates all generated code, then performs an incremental tsc build.
echo -e "${BLUE}${BOLD}Attempting fast incremental build...${RESET}"
echo
yarn install --immutable

if ! yarn build:fast; then
  echo -e "${YELLOW}${BOLD}Incremental build failed for some reason, attempting full build...${RESET}"
  echo
  yarn build
fi

echo
echo -e "${GREEN}Yarn project successfully built!${RESET}"
