#!/bin/bash
set -eux

MAX_WAIT_TIME=300 # Maximum wait time in seconds
WAIT_INTERVAL=10  # Interval between checks in seconds
elapsed_time=0

exec &> >(tee -a ~/.maybe-exit-log)

# we have this in a minutely crontab for simplicity, but we only want one to run
if [ -f ~/.maybe-exit-spot-lock ] ; then
  echo "Already running maybe_exit_spot.sh"
  exit
fi

exec >~/.maybe-exit-spot-log

cleanup() {
  rm ~/.maybe-exit-spot-lock
}

trap cleanup EXIT
touch ~/.maybe-exit-spot-lock

# We wait to see if a runner comes up in
while ! pgrep Runner.Worker > /dev/null && ! pgrep earthly > /dev/null ; do
  if [ $elapsed_time -ge $MAX_WAIT_TIME ]; then
    echo "Found no runner or earthly instance for $MAX_WAIT_TIME, shutting down now."
    ~/spot_runner_graceful_exit.sh
    sudo shutdown now
    exit
  fi

  sleep $WAIT_INTERVAL
  elapsed_time=$((elapsed_time + WAIT_INTERVAL))
done
echo "System seems alive, doing nothing."