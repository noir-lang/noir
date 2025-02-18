#!/bin/bash
set -eu

# This file is used for Noir CI and is not required.

BACKEND=${BACKEND:-bb}

rm ./Oracle.*

./log_and_exec_transcript.sh