#!/bin/bash
set -eu

cd $(dirname $0)

rm -rf bin share

[ "${TARGETARCH:-$(uname -m)}" = "arm64" ] && echo "export CFLAGS=-mno-outline-atomics" >> $HOME/.profile || true

if [ ! -d './foundry-repo' ]; then
    git clone --depth 1 --branch nightly-de33b6af53005037b463318d2628b5cfcaf39916 \
        https://github.com/foundry-rs/foundry.git foundry-repo
fi

(cd foundry-repo && cargo build --profile local)

mkdir bin \
    && mv foundry-repo/target/local/forge bin/forge \
    && mv foundry-repo/target/local/cast bin/cast \
    && mv foundry-repo/target/local/anvil bin/anvil \
    && mv foundry-repo/target/local/chisel bin/chisel \
    && strip bin/forge \
    && strip bin/cast \
    && strip bin/chisel \
    && strip bin/anvil;