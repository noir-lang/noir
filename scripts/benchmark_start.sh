#!/bin/bash

echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
