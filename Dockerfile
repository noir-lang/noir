FROM noir-env:latest as builder

WORKDIR /build
COPY . src

RUN cargo --version
RUN cd /build/src/crates/nargo; cargo build --release
RUN echo $HOME

FROM ubuntu:20.04
RUN apt-get update && apt-get install -y --no-install-recommends \
        libssl-dev \
        libomp-dev

WORKDIR /home/noir

ENV NARGO_HOME "/home/noir/.nargo"
COPY --from=builder /build/src/target/release/nargo $NARGO_HOME/bin/
COPY --from=builder /root/.config/noir-lang /root/.config/noir-lang
COPY --from=builder /build/src/crates/nargo/tests/test_data $NARGO_HOME/examples

ENV PATH "$NARGO_HOME/bin:$PATH"