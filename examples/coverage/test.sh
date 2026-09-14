#!/bin/bash
set -eu

cd $(dirname $0)

rm -rf target

./coverage_report.sh
