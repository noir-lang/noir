FROM alpine:3.18 as build-environment

ARG TARGETARCH
WORKDIR /opt

RUN apk add clang lld curl build-base linux-headers git \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh \
    && chmod +x ./rustup.sh \
    && ./rustup.sh -y

RUN [[ "$TARGETARCH" = "arm64" ]] && echo "export CFLAGS=-mno-outline-atomics" >> $HOME/.profile || true

WORKDIR /opt
RUN git clone --depth 1 --branch nightly-de33b6af53005037b463318d2628b5cfcaf39916 https://github.com/foundry-rs/foundry.git
WORKDIR /opt/foundry

RUN source $HOME/.profile && cargo build --release \
    && mkdir out \
    && mv target/release/forge out/forge \
    && mv target/release/cast out/cast \
    && mv target/release/anvil out/anvil \
    && mv target/release/chisel out/chisel \
    && strip out/forge \
    && strip out/cast \
    && strip out/chisel \
    && strip out/anvil;

FROM alpine:3.18 as foundry-client
RUN apk add --no-cache linux-headers git
COPY --from=build-environment /opt/foundry/out/forge /usr/local/bin/forge
COPY --from=build-environment /opt/foundry/out/cast /usr/local/bin/cast
COPY --from=build-environment /opt/foundry/out/anvil /usr/local/bin/anvil
COPY --from=build-environment /opt/foundry/out/chisel /usr/local/bin/chisel
RUN adduser -Du 1000 foundry
ENTRYPOINT ["/bin/sh", "-c"]