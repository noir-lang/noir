#!/usr/bin/env bash
# Usage:
# Bootstraps the repo. End to end tests should be runnable after a bootstrap:
#   ./bootstrap.sh
# Run a second time to perform a "light bootstrap", rebuilds code that's changed:
#   ./bootstrap.sh
# Force a clean of the repo before performing a full bootstrap, erases untracked files, be careful!
#   ./bootstrap.sh clean
set -eu

cd "$(dirname "$0")"

CMD=${1:-}

YELLOW="\033[93m"
BOLD="\033[1m"
RESET="\033[0m"

source ./build-system/scripts/setup_env '' '' '' > /dev/null

if [ "$CMD" = "clean" ]; then
  echo "WARNING: This will erase *all* untracked files, including hooks and submodules."
  echo -n "Continue? [y/n] "
  read user_input
  if [ "$user_input" != "y" ] && [ "$user_input" != "Y" ]; then
    exit 1
  fi

  # Remove hooks and submodules.
  rm -rf .git/hooks/*
  rm -rf .git/modules/*
  for SUBMODULE in $(git config --file .gitmodules --get-regexp path | awk '{print $2}'); do
    rm -rf $SUBMODULE
  done

  # Remove all untracked files, directories, nested repos, and .gitignore files.
  git clean -ffdx

  exit 0
elif [ "$CMD" = "full" ]; then
  if can_use_ci_cache; then
    echo -e "${BOLD}${YELLOW}WARNING: Performing a full bootstrap. Consider leveraging './bootstrap.sh fast' to use CI cache.${RESET}"
    echo
  fi
elif [ "$CMD" = "fast" ]; then
  export USE_CACHE=1
  if ! can_use_ci_cache; then
    echo -e "${BOLD}${YELLOW}WARNING: Either docker or aws credentials are missing. Install docker and request credentials. Note this is for internal aztec devs only.${RESET}"
    exit 1
  fi
else
  echo "usage: $0 <full|fast|clean>"
  exit 1
fi

# Install pre-commit git hooks.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "(cd barretenberg/cpp && ./format.sh staged)" >$HOOKS_DIR/pre-commit
chmod +x $HOOKS_DIR/pre-commit

git submodule update --init --recursive

PROJECTS=(
  barretenberg
  noir
  foundry
  l1-contracts
  avm-transpiler
  noir-projects
  yarn-project
)

# Build projects locally
for P in "${PROJECTS[@]}"; do
  echo "**************************************"
  echo -e "\033[1mBootstrapping $P...\033[0m"
  echo "**************************************"
  echo
  (cd $P && ./bootstrap.sh)
  echo
  echo
done
