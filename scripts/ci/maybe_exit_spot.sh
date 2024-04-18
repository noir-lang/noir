#!/bin/bash
set -eux

MAX_WAIT_TIME=300 # Maximum wait time in seconds
WAIT_INTERVAL=10  # Interval between checks in seconds
elapsed_time=0

exec &> >(tee -a /run/.maybe-exit-log)

# we have this in a minutely crontab for simplicity, but we only want one to run
if [ -f /run/.maybe-exit-spot-lock ] ; then
  echo "Already running maybe_exit_spot.sh"
  exit
fi

exec >/run/.maybe-exit-spot-log

cleanup() {
  rm /run/.maybe-exit-spot-lock
}

trap cleanup EXIT
touch /run/.maybe-exit-spot-lock

# We wait to see if a runner comes up in
while ! pgrep Runner.Worker > /dev/null; do
  if [ $elapsed_time -ge $MAX_WAIT_TIME ]; then
    echo "Found no runner for $MAX_WAIT_TIME, shutting down now."
    /run/spot_runner_graceful_exit.sh
    shutdown now
    exit
  fi

  sleep $WAIT_INTERVAL
  elapsed_time=$((elapsed_time + WAIT_INTERVAL))
done
echo "System seems alive, doing nothing."