#!/usr/bin/env bash
set -eu

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
  yarn tsc -b tsconfig.json --watch &
  TSC_PID=$!
  echo "$TSC_PID" > .tsc.pid
}

# Stops the typescript watch process
stop_tsc_watch() {
  local tsc_pid=$(cat ".tsc.pid");
  kill $tsc_pid || true
}

# Kill typescript, run a yarn generate, and restart typescript
run_generate() {
  echo "Change detected at $1"
  stop_tsc_watch
  FORCE_COLOR=true yarn workspaces foreach --parallel --topological-dev --verbose run generate:$1
  echo "Generate complete, restarting typescript..."
  sleep 3
  start_tsc_watch
}

# Remove all temp files with process or run ids on exit
cleanup() {
  rm .tsc.pid || true
  rm .debounce-* || true
}
trap cleanup EXIT

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
        echo "Change at $folder not matched with any project"
        exit 1
        ;;
    esac
done



