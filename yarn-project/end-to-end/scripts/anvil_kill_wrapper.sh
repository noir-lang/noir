#!/bin/bash

# Function to get the PPID in macOS
get_ppid_macos() {
  ps -j $$ | awk 'NR==2 {print $3}'
}

# Function to get the PPID in Linux
get_ppid_linux() {
  awk '{print $4}' /proc/$$/stat
}

# Function to check if a process is alive in macOS
is_process_alive_macos() {
  ps -p $1 > /dev/null 2>&1
}

# Function to check if a process is alive in Linux
is_process_alive_linux() {
  [ -d /proc/$1 ]
}


# Determine the operating system and call the appropriate function
if [[ "$OSTYPE" == "darwin"* ]]; then
  PARENT_PID=$(get_ppid_macos)
  check_process_alive() { is_process_alive_macos $1; }
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PARENT_PID=$(get_ppid_linux)
  check_process_alive() { is_process_alive_linux $1; }
else
  echo "Unsupported OS"
  exit 1
fi

# echo "Parent PID: $PARENT_PID"

# Start anvil in the background.
anvil $@ &
CHILD_PID=$!

cleanup() {
    kill $CHILD_PID
}

trap cleanup EXIT

# Continuously check if the parent process is still alive.
while check_process_alive $PARENT_PID; do
  sleep 1
done
