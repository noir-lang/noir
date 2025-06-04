#!/bin/bash
set -eu

cd $(dirname $0)

# This file is used for Noir CI and is not required.

rm -f ./Oracle.*

./log_and_exec_transcript.sh
