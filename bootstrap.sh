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

if [ -n "$CMD" ]; then
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
  else
    echo "Unknown command: $CMD"
    exit 1
  fi
fi

# if [ ! -f ~/.nvm/nvm.sh ]; then
#   echo "Nvm not found at ~/.nvm"
#   exit 1
# fi

# Install pre-commit git hooks.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "(cd barretenberg/cpp && ./format.sh staged)" > $HOOKS_DIR/pre-commit
chmod +x $HOOKS_DIR/pre-commit

git submodule update --init --recursive

PROJECTS=(
  barretenberg
  noir
  l1-contracts
  yarn-project
)

# Build projects locally
for P in "${PROJECTS[@]}"; do
  if [ -n "${BOOTSTRAP_USE_REMOTE_CACHE:-}" ] && [ -f "$P/bootstrap_cache.sh" ]; then
    echo "**************************************"
    echo -e "\033[1mBootstrapping $P from remote cache...\033[0m"
    echo "**************************************"
    echo
    $P/bootstrap_cache.sh
  else  
    echo "**************************************"
    echo -e "\033[1mBootstrapping $P...\033[0m"
    echo "**************************************"
    echo
    $P/bootstrap.sh
  fi
  echo
  echo
done
