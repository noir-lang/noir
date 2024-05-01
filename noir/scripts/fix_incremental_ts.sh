#!/usr/bin/env bash
# Hack to workaround incremental builds not working on mac when mounted into dev container.
# Drops the fractional part of file timestamps.
set -eu

cd $(dirname $0)/../noir-repo

if [ "${HOST_OSTYPE:-}" == "darwin" ]; then
  echo -n "Fixing incremental timestamps... "
  find target -type f -print0 | xargs -0 -P $(nproc) -I {} sh -c 'touch -d @$(stat --format="%Y" {}) {}'
  echo "Done."
fi