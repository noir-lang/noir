#!/usr/bin/env bash
# Usage: ./bootstrap.sh <full|fast|check|clean>"
#   full: Bootstrap the repo from scratch.
#   fast: Bootstrap the repo using CI cache where possible to save time building.
#   check: Check required toolchains and versions are installed.
#   clean: Force a complete clean of the repo. Erases untracked files, be careful!
set -eu

if [ "$(uname)" == "Darwin" ]; then
  shopt -s expand_aliases
  alias clang++-16="clang++"
fi

cd "$(dirname "$0")"

CMD=${1:-}

YELLOW="\033[93m"
RED="\033[31m"
BOLD="\033[1m"
RESET="\033[0m"

source ./build-system/scripts/setup_env '' '' '' > /dev/null

function encourage_dev_container {
  echo -e "${BOLD}${RED}ERROR: Toolchain incompatability. We encourage use of our dev container. See build-images/README.md.${RESET}"
}

# Checks for required utilities, toolchains and their versions.
# Developers should probably use the dev container in /build-images to ensure the smoothest experience.
function check_toolchains {
  # Check for various required utilities.
  for util in jq parallel awk git curl; do
    if ! command -v $util > /dev/null; then
      encourage_dev_container
      echo "Utility $util not found."
      exit 1
    fi
  done
  # Check cmake version.
  CMAKE_MIN_VERSION="3.24"
  CMAKE_INSTALLED_VERSION=$(cmake --version | head -n1 | awk '{print $3}')
  if [[ "$(printf '%s\n' "$CMAKE_MIN_VERSION" "$CMAKE_INSTALLED_VERSION" | sort -V | head -n1)" != "$CMAKE_MIN_VERSION" ]]; then
    encourage_dev_container
    echo "Minimum cmake version 3.24 not found."
    exit 1
  fi
  # Check clang version.
  if ! clang++-16 --version > /dev/null; then
    encourage_dev_container
    echo "clang 16 not installed."
    echo "Installation: sudo apt install clang-16"
    exit 1
  fi
  # Check rust version.
  if ! rustup show | grep "1.74" > /dev/null; then
    encourage_dev_container
    echo "Rust version 1.74 not installed."
    echo "Installation:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.74.1"
    exit 1
  fi
  # Check wasi-sdk version.
  if ! cat /opt/wasi-sdk/VERSION 2> /dev/null | grep 22.0 > /dev/null; then
    encourage_dev_container
    echo "wasi-sdk-22 not found at /opt/wasi-sdk."
    echo "Use dev container, build from source, or you can install linux x86 version with:"
    echo "  curl -s -L https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-22/wasi-sdk-22.0-linux.tar.gz | tar zxf - && sudo mv wasi-sdk-22.0 /opt/wasi-sdk"
    exit 1
  fi
  # Check foundry version.
  for tool in forge anvil; do
    if ! $tool --version 2> /dev/null | grep de33b6a > /dev/null; then
      encourage_dev_container
      echo "$tool not in PATH or incorrect version (requires de33b6af53005037b463318d2628b5cfcaf39916)."
      echo "Installation: https://book.getfoundry.sh/getting-started/installation (requires rust 1.75)"
      echo "  curl -L https://foundry.paradigm.xyz | bash"
      echo "  foundryup -b de33b6af53005037b463318d2628b5cfcaf39916"
      exit 1
    fi
  done
  # Check Node.js version.
  NODE_MIN_VERSION="18.19.0"
  NODE_INSTALLED_VERSION=$(node --version | cut -d 'v' -f 2)
  if [[ "$(printf '%s\n' "$NODE_MIN_VERSION" "$NODE_INSTALLED_VERSION" | sort -V | head -n1)" != "$NODE_MIN_VERSION" ]]; then
    encourage_dev_container
    echo "Minimum Node.js version 18.19.0 not found."
    echo "Installation: nvm install 18"
    exit 1
  fi
  # Check for required npm globals.
  for util in yarn solhint; do
    if ! command -v $util > /dev/null; then
      encourage_dev_container
      echo "$util not found."
      echo "Installation: npm install --global $util"
      exit 1
    fi
  done
}

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
elif [ "$CMD" = "check" ]; then
  check_toolchains
  echo "Toolchains look good! 🎉"
  exit 0
else
  echo "usage: $0 <full|fast|check|clean>"
  exit 1
fi

# Install pre-commit git hooks.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "(cd barretenberg/cpp && ./format.sh staged)" >$HOOKS_DIR/pre-commit
chmod +x $HOOKS_DIR/pre-commit

git submodule update --init --recursive

check_toolchains

PROJECTS=(
  barretenberg
  noir
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
