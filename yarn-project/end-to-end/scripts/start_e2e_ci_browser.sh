#!/bin/sh

TEST=${1:-./src/e2e_aztec_js_browser.test.ts}

apk add dbus

# Create dbus dirs
mkdir -p /var/run/dbus

# Change ownership and permissions if necessary
chown -R root:root /var/run/dbus
chmod -R 755 /var/run/dbus

dbus-daemon --system --nofork &
yarn test $TEST
