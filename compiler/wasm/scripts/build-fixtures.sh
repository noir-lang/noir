#!/usr/bin/env bash

nargo compile --program-dir ./test/fixtures/simple --pedantic-solving
nargo compile --program-dir ./test/fixtures/with-deps --pedantic-solving
nargo compile --program-dir ./test/fixtures/noir-contract --pedantic-solving
