#!/bin/bash
set -eu

apt-get install libc++-dev -y
yarn workspace integration-tests test
