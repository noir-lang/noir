#!/bin/bash

echo 4 | sudo tee /proc/sys/kernel/perf_event_paranoid
