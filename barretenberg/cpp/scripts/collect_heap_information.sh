#!/usr/bin/env bash
set -eu

PRESET=gperftools
ONLY_PROCESS=${1:-}
EXECUTABLE=${2:-ultra_honk_rounds_bench}

# Move above script dir.
cd $(dirname $0)/..

# Configure and build with heap profiling preset.

cmake --preset $PRESET
cmake --build --preset $PRESET

cd build-$PRESET

if [ -z "$ONLY_PROCESS" ]; then
  # Clear old heap profile data.
  rm -f $EXECUTABLE.heap*

  # Run application with heap profiling to a file with prefix '$EXECUTABLE'.
  HEAPPROFILE=./$EXECUTABLE ./bin/$EXECUTABLE
fi

# Download and install Go
if [ ! -d ~/go ]; then
  ARCHIVE=go1.21.3.linux-amd64.tar.gz
  echo "Downloading and installing Go..."
  wget https://go.dev/dl/$ARCHIVE
  tar -C ~/ -xvf $ARCHIVE
  rm $ARCHIVE
  export PATH=$PATH:~/go/bin
fi

# Install pprof
if [ ! -f ~/go/bin/pprof ]; then
    echo "Installing pprof..."
    ~/go/bin/go install github.com/google/pprof@latest
fi

# Collect the heap files
files=(./$EXECUTABLE.*.heap)
# Find the middle index based on the count
middle_index=$(( (${#files[@]} + 1) / 2 - 1))
# Process the heap profile with pprof
~/go/bin/pprof --text ./bin/$EXECUTABLE ${files[$middle_index]}
