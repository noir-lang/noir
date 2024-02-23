# NOTE: This script is NOT meant to be ran, only sourced.
# This sets up all the necessary machinery to lock ~/BENCHMARK_IN_PROGRESS
# 

# Function to clean up lock file
function cleanup() {
    ssh $BB_SSH_KEY $BB_SSH_INSTANCE "rm -f ~/BENCHMARK_IN_PROGRESS"
    echo "Benchmarking lock deleted."
}

# Check for existing lock file
if ssh $BB_SSH_KEY $BB_SSH_INSTANCE "test -f ~/BENCHMARK_IN_PROGRESS"; then
    echo "Benchmarking is already in progress. If htop on the remote machine is not active, ~/BENCHMARK_IN_PROGRESS may need to be deleted."
    # Important: Exits the script that called this!
    # This implements the lock
    exit 1
fi

# Create lock file
ssh $BB_SSH_KEY $BB_SSH_INSTANCE "touch ~/BENCHMARK_IN_PROGRESS"

echo "Benchmarking lock created at ~/BENCHMARK_IN_PROGRESS."

# Trap to ensure cleanup runs on ANY exit, including from a signal
trap cleanup EXIT
trap cleanup INT # handle ctrl-c

# don't exit, the caller script will run