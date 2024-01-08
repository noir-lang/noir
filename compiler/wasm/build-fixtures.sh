#!/usr/bin/env bash

cd test/fixtures && cd simple && nargo compile && cd ../with-deps && nargo compile && cd ../../