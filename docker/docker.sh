#!/usr/bin/env bash
docker build --tag nargo .
docker run nargo $@
