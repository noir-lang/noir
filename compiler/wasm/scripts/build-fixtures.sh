#!/usr/bin/env bash

nargo compile --program-dir ./test/fixtures/simple
nargo compile --program-dir ./test/fixtures/with-deps
nargo compile --program-dir ./test/fixtures/noir-contract