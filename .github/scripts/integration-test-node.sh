#!/bin/bash
set -eu

apt-get install libc6 libstdc++6 -y
yarn workspace integration-tests test:node
