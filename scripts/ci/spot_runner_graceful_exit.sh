# Adapted from https://github.com/actions/actions-runner-controller/blob/master/runner/graceful-stop.sh
#!/bin/bash

set -eu

export RUNNER_ALLOW_RUNASROOT=1
# This should be short so that the job is cancelled immediately, instead of hanging for 10 minutes or so and failing without any error message.
RUNNER_GRACEFUL_STOP_TIMEOUT=${RUNNER_GRACEFUL_STOP_TIMEOUT:-15}

echo "Executing graceful shutdown of github action runners."

# The below procedure atomically removes the runner from GitHub Actions service,
# to ensure that the runner is not running any job.
# This is required to not terminate the actions runner agent while running the job.
# If we didn't do this atomically, we might end up with a rare race where
# the runner agent is terminated while it was about to start a job.

# glob for all our installed runner directories
for RUNNER_DIR in /run/*-ec2-* ; do
  pushd $RUNNER_DIR
  ./config.sh remove --token "$(cat $RUNNER_DIR/.runner-token)" || true &
  popd
done
wait

if pgrep Runner.Listener > /dev/null; then
  # The below procedure fixes the runner to correctly notify the Actions service for the cancellation of this runner.
  # It enables you to see `Error: The operation was canceled.` vs having it hang for 10 minutes or so.
  kill -TERM $(pgrep Runner.Listener)
  while pgrep Runner.Listener > /dev/null; do
    sleep 1
  done
fi
echo "Cleaning up lingering runner registrations."
for RUNNER_DIR in /run/*-ec2-* ; do
  pushd $RUNNER_DIR
  while [ -f .runner ] ; do
    ./config.sh remove --token "$(cat $RUNNER_DIR/.runner-token)" || true
    sleep 1
  done
  popd
done
echo "Graceful github runner stop completed."