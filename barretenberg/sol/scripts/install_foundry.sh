#!/bin/sh
set -eu

export FOUNDRY_DIR="$PWD/.foundry"
FOUNDRY_BIN_DIR="$FOUNDRY_DIR/bin"
BIN_URL="https://raw.githubusercontent.com/foundry-rs/foundry/master/foundryup/foundryup"
BIN_PATH="$FOUNDRY_BIN_DIR/foundryup"
FOUNDRY_MAN_DIR="$FOUNDRY_DIR/share/man/man1"

# Clean
rm -rf $FOUNDRY_DIR

# Install foundryup.
mkdir -p $FOUNDRY_BIN_DIR
mkdir -p $FOUNDRY_MAN_DIR
curl -# -L $BIN_URL -o $BIN_PATH
chmod +x $BIN_PATH
export PATH=$FOUNDRY_BIN_DIR:$PATH

# Use version.
foundryup
