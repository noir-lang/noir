#!/usr/bin/env bash
set -u

DEBOUNCE_DURATION=3 # Set a high duration for debounce since nargo build may pause for a long time during a compilation
INOTIFY_EVENTS="modify,create,delete,move"
NOIR_CONTRACTS_OUT_DIR="../noir-projects/noir-contracts/target/"
NOIR_CIRCUITS_OUT_DIR="../noir-projects/noir-protocol-circuits/target/"
L1_CONTRACTS_OUT_DIR="../l1-contracts/out/"

# Debounce any command sent here. Grouped by command name and first arg.
debounce() {
  local group_id="$1-$2"
  local run_id=$(uuidgen)
  echo "$run_id" > ".debounce-$group_id"
  (
    sleep $DEBOUNCE_DURATION; 
    local current_id=$(cat ".debounce-$group_id");
    if [ "$run_id" = "${current_id}" ]; then
      "$@"
    fi
  ) &
}

# Start typescript watch process in the background and store process ID in a file
start_tsc_watch() {
  local tsc_bin=$(yarn bin tsc)
  $tsc_bin -b tsconfig.json --watch &
  TSC_PID=$!
  echo "$TSC_PID" > .tsc.pid
}

# Stops the typescript watch process
stop_tsc_watch() {
  if [ -f .tsc.pid ]; then
    echo "Stopping tsc watch..."
    local tsc_pid=$(cat ".tsc.pid");
    echo KILLING $tsc_pid
    kill $tsc_pid
  fi
}

# Kill typescript, run a yarn generate, and restart typescript
run_generate() {
  # If already generating something, then try again in a few seconds
  if [ -f .generating.lock ]; then
    debounce run_generate $1
    return
  fi
  # Pause ts watch to generate code and acquire a lock
  echo "LOCKED" > .generating.lock
  echo "Change detected at $1..."
  stop_tsc_watch
  if FORCE_COLOR=true yarn workspaces foreach --parallel --topological-dev --verbose run generate:$1; then
    echo "Generate complete, restarting tsc watch..."
  else
    echo "Generate failed, restarting tsc watch..."
  fi
  # Restart tsc watch and release lock
  sleep 3
  start_tsc_watch
  rm .generating.lock
}

# Remove all temp files with process or run ids on exit
cleanup() {
  rm -f .tsc.pid || true
  rm -f .debounce-* || true
  rm -f .generating.lock || true
}
trap cleanup EXIT
cleanup

# Start tsc watch in background
start_tsc_watch

# Watch for changes in the output directories
while true; do
    folder=$(inotifywait --format '%w' --quiet --recursive --event $INOTIFY_EVENTS $NOIR_CONTRACTS_OUT_DIR $NOIR_CIRCUITS_OUT_DIR $L1_CONTRACTS_OUT_DIR)
    case $folder in
      "$NOIR_CONTRACTS_OUT_DIR")
        debounce run_generate "noir-contracts"
        ;;
      "$NOIR_CIRCUITS_OUT_DIR")
        debounce run_generate "noir-circuits"
        ;;
      "$L1_CONTRACTS_OUT_DIR"*)
        debounce run_generate "l1-contracts"
        ;;
      *)
        echo "Error: change at $folder not matched with any project"
        exit 1
        ;;
    esac
done



