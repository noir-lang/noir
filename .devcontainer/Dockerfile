FROM ubuntu:noble AS base-build
RUN export DEBIAN_FRONTEND="noninteractive" \
    && apt update && apt install --no-install-recommends -y \
    build-essential \
    ca-certificates \
    bash \
    clang \
    cmake \
    make \
    ninja-build \
    git \
    curl \
    python3 \
    wget \
    && apt-get -y autoremove \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

########################################################################################################################
# Build foundry.
FROM base-build AS foundry
ENV FOUNDRY_TAG=25f24e677a6a32a62512ad4f561995589ac2c7dc
ENV FOUNDRY_BIN_DIR="/root/.foundry/bin"
ENV RUSTUP_HOME=/opt/rust/rustup
ENV CARGO_HOME=/opt/rust/cargo
ENV PATH="/opt/rust/cargo/bin:$PATH"
ENV RUSTFLAGS="-C target-cpu=generic"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN curl -L https://foundry.paradigm.xyz | bash \
    && $FOUNDRY_BIN_DIR/foundryup -b nightly-$FOUNDRY_TAG \
    && mkdir -p /opt/foundry/bin \
    && for t in forge cast anvil chisel; do \
    mv $FOUNDRY_BIN_DIR/$t /opt/foundry/bin/$t; \
    strip /opt/foundry/bin/$t; \
    done \
    && rm -rf $FOUNDRY_BIN_DIR

########################################################################################################################
# Build the build image.
# This image contains *just* what's needed to perform a full build of the aztec project and to run individual tests.
# This is not sufficient for development, as it lacks many useful developer tools.
# This is not sufficient for testing everything as per in CI, as that requires docker-in-docker.
# You should use the devbox image for development and CI runs.
# We favour using the devbox in CI as we can drop into a familiar environment on error.
# It also means we're effectively testing the devbox in CI.
# We use this image for test isolation.
FROM base-build AS build
RUN apt update && \
    apt install -y \
    # Utils
    curl \
    git \
    wget \
    jq \
    gawk \
    unzip \
    # Node (18.19.1)
    nodejs \
    npm \
    # Unminimize ubuntu installation.
    unminimize \
    && apt-get -y autoremove \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Install foundry.
COPY --from=foundry /opt/foundry /opt/foundry
ENV PATH="/opt/foundry/bin:$PATH"

# Install rust and cross-compilers. Noir specifically uses 1.85.0.
# We give everyone write ownership so downstream boxes can write.
ENV RUSTUP_HOME=/opt/rust/rustup
ENV CARGO_HOME=/opt/rust/cargo
ENV PATH="/opt/rust/cargo/bin:$PATH"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.85.0 && \
    rustup target add wasm32-unknown-unknown wasm32-wasip1 aarch64-apple-darwin && \
    rustup component add rust-src && \
    chmod -R a+w /opt/rust

# Install cargo binstall
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

RUN cargo binstall wasm-bindgen-cli@0.2.100 -y
RUN cargo-binstall wasm-opt@0.116.0 -y
RUN cargo-binstall wasm-pack@0.12.1 -y
RUN cargo-binstall cargo-nextest@0.9.67 -y --secure

# Install corepack and yarn.
RUN npm install --global corepack && corepack enable && corepack install --global yarn@4.5.2

# Install yq
RUN curl -sL https://github.com/mikefarah/yq/releases/download/v4.42.1/yq_linux_$(dpkg --print-architecture) \
    -o /usr/local/bin/yq && chmod +x /usr/local/bin/yq

# Install playwright for browser testing. Ensure accessible for other users.
RUN npx -y playwright@1.49 install --with-deps && mv /root/.cache/ms-playwright /opt/ms-playwright
ENV PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright

########################################################################################################################
# We want to produce downstream images: devbox and sysbox. This image is the base image for each.
# It contains a suite of tools that developers might use to develop aztec.
FROM build AS basebox
RUN yes | unminimize

# Install stuff devs need.
RUN apt update && \
    apt install -y \
        cgroup-tools \
        gnupg \
        htop \
        iproute2 \
        iputils-ping \
        less \
        libfuse2 \
        lldb \
        lsb-release \
        man \
        neovim \
        psmisc \
        python3-blessed \
        redis-tools \
        rsync \
        software-properties-common \
        ssh \
        strace \
        sudo \
        telnet \
        tmux \
        vim \
        zsh \
        inotify-tools \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*


# Install gh (github cli).
RUN mkdir -p -m 755 /etc/apt/keyrings && wget -qO- https://cli.github.com/packages/githubcli-archive-keyring.gpg > /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt update \
    && apt install gh -y

# Sets LANG explicitly. Ensures tmux shows unicode symbols.
ENV LANG=C.UTF-8
# Ensure we get color terminal.
ENV TERM=xterm-256color

# sudo group can sudo without password.
RUN echo '%sudo ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers
