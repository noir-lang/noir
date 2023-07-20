#!/bin/bash
set -eu

SCRIPT_DIR=$(dirname "$(realpath "$0")")
"$SCRIPT_DIR"/git-subrepo/lib/git-subrepo $@

