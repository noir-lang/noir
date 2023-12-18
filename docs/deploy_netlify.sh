#!/usr/bin/env bash
[ -n "${BUILD_SYSTEM_DEBUG:-}" ] && set -x # conditionally trace
set -eu

extract_repo docs /usr/src extracted-repo
cd extracted-repo/src/docs
npm install netlify-cli -g
netlify deploy --site aztec-docs-dev
netlify deploy --site aztec-docs-dev --prod