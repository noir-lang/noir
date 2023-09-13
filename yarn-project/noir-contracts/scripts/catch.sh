#!/bin/bash

# Handler for SIGCHLD, cleanup if child exit with error, used by nargo_test.sh and compile.sh
handle_sigchld() {
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