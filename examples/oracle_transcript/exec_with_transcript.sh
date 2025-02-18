#!/bin/bash
set -eu

dir=$(dirname $0)

nargo --program-dir $dir test