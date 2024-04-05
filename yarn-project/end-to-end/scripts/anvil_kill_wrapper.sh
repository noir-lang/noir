#!/bin/bash

# Find the parent of this script.
PARENT_PID=$(awk '{print $4}' /proc/$$/stat)

# Start anvil in the background.
../../foundry/bin/anvil $@ &
CHILD_PID=$!

cleanup() {
    kill $CHILD_PID
}

trap cleanup EXIT

# Continuously check if the parent process is still alive.
while [ -d /proc/$PARENT_PID ]; do
    sleep 1
done