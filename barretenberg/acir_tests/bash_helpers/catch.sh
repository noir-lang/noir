#!/usr/bin/env bash

# Handler for SIGCHLD, cleanup if child exit with error
handle_sigchild() {
    for pid in "${pids[@]}"; do
        # If process is no longer running
        if ! kill -0 "$pid" 2>/dev/null; then
            # Wait for the process and get exit status
            wait "$pid"
            status=$?

            # If exit status is error
            if [ $status -ne 0 ]; then
                # Create error file
                touch "$error_file"
            fi
        fi
    done
}

check_error_file() {
    # If error file exists, exit with error
    if [ -f "$error_file" ]; then
        rm "$error_file"
        echo "Error occurred in one or more child processes. Exiting..."
        exit 1
    fi
}