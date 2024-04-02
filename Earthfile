VERSION 0.8
PROJECT noir-lang/noir
IMPORT github.com/earthly/lib/rust:3.0.2 AS rust

FROM node:18.19.1

# Copied from https://github.com/rust-lang/docker-rust/blob/95dfe05d230fddf4f89bd3df46086eed15e0c832/1.73.0/bullseye/Dockerfile

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install the version of Rust as described in `rust-toolchain.toml`
COPY ./rust-toolchain.toml .
ENV RUST_VERSION=$(grep '^channel =' ./rust-toolchain.toml | sed -E 's/channel = "([^"]+)"/\1/')

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='0b2f6c8f85a3d02fde2efc0ced4657869d73fccfce59defb4e8d29233116e6db' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='f21c44b01678c645d8fbba1e55e4180a01ac5af2d38bcbd14aa665e0d96ed69a' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='673e336c81c65e6b16dcdede33f4cc9ed0f08bde1dbe7a935f113605292dc800' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e7b0f47557c1afcd86939b118cbcf7fb95a5d1d917bdd355157b63ca00fc4333' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.26.0/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

# Initialize earthly library for cached rust builds.
RUN cargo install cargo-sweep@0.7.0 --locked --root $CARGO_HOME
DO rust+INIT --keep_fingerprints=true

# Install various tools used for building JS packages
RUN apt-get update && apt-get install --no-install-recommends -qq jq libc++1

git-info:
    LOCALLY
    RUN mkdir -p ./tmp
    RUN git rev-parse --verify HEAD > ./tmp/commit_hash
    RUN (git diff --exit-code --quiet; test $? -eq 0 && echo "false" || echo "true") > ./tmp/dirty

    SAVE ARTIFACT ./tmp/commit_hash commit_hash
    SAVE ARTIFACT ./tmp/dirty dirty
    RUN rm -rf ./tmp
   
source:
    # TODO: we're pulling in a lot of non-rust source here, e.g. READMEs.
    WORKDIR ./project
    COPY --keep-ts Cargo.toml Cargo.lock rust-toolchain.toml .rustfmt.toml ./
    COPY --keep-ts --dir acvm-repo/acir acvm-repo/acir_field acvm-repo/acvm acvm-repo/acvm_js acvm-repo/blackbox_solver acvm-repo/bn254_blackbox_solver acvm-repo/brillig acvm-repo/brillig_vm ./acvm-repo
    COPY --keep-ts --dir compiler/fm compiler/noirc_driver compiler/noirc_errors compiler/noirc_evaluator compiler/noirc_frontend compiler/noirc_printable_type compiler/utils compiler/wasm ./compiler
    COPY --keep-ts --dir tooling/acvm_cli tooling/backend_interface tooling/bb_abstraction_leaks tooling/debugger tooling/lsp tooling/nargo tooling/nargo_cli tooling/nargo_fmt tooling/nargo_toml tooling/noirc_abi tooling/noirc_abi_wasm ./tooling
    COPY --keep-ts --dir aztec_macros noir_stdlib test_programs ./

    DO rust+CARGO --args=fetch

    SAVE ARTIFACT --keep-ts ./*

build:
    FROM +source
    ARG GIT_COMMIT
    IF [ -n "${GIT_COMMIT}" ]
        ENV GIT_COMMIT=$GIT_COMMIT
        ENV GIT_DIRTY="false"
    ELSE
        COPY --dir +git-info/* /tmp/git/
        ENV GIT_COMMIT=$(cat /tmp/git/commit_hash)
        ENV GIT_DIRTY=$(cat /tmp/git/dirty)
        RUN rm -rf /tmp/git
    END

    DO rust+CARGO --args="build --offline --release" --output="release/nargo"
    SAVE ARTIFACT ./target/release/nargo

build-msrv:
    FROM +source
    # We force the ACVM crate and all of its dependencies to update their dependencies
    # This ensures that we'll be able to build the crates when they're being published. 
    DO rust+CARGO --args="update --package acvm --aggressive"
    DO rust+CARGO --args="update --package bn254_blackbox_solver --aggressive"
    DO rust+CARGO --args="fetch"

    DO rust+CARGO --args="build --offline --release" --output="release/nargo"
    
publish-builds:
    COPY (+cross-build/nargo-x86_64-unknown-linux-gnu.tar.gz --target="x86_64-unknown-linux-gnu") ./release-tarballs/
    COPY (+cross-build/nargo-x86_64-unknown-linux-musl.tar.gz --target="x86_64-unknown-linux-musl") ./release-tarballs/
    SAVE ARTIFACT ./release-tarballs/

cross-build:
    FROM +source
    ARG --required target
    RUN apt install p7zip-full -y --no-install-recommends
    DO rust+CROSS --args="build --offline --release" --target ${target} --output=${target}/release/nargo
    RUN 7z a -ttar -so -an ./target/${target}/release/nargo | 7z a -si ./nargo-${target}.tar.gz
    SAVE ARTIFACT ./nargo-${target}.tar.gz

test:
    # TODO: export a nextest archive and parallelise running of tests
    FROM +source
    DO rust+CARGO --args="test --release --workspace"

# Format Rust source code
fmt:
    FROM +source
    DO rust+CARGO --args="fmt --check --all"

# Format Noir source code
nargo-fmt:
    FROM +source
    COPY --dir +build/nargo ./target/release/nargo
    ENV PATH=/project/target/release:$PATH

    RUN cd noir_stdlib && nargo fmt --check
    RUN cd test_programs && ./format.sh check

clippy:
    FROM +source
    # Treat warnings as errors.
    ENV RUSTFLAGS=-Dwarnings
    DO rust+CARGO --args="clippy --release --workspace --all-targets"
  
# Pull in `package.json`s for yarn workspace and install all dependencies for caching
yarn-deps:
    WORKDIR ./project
    COPY --dir --keep-ts .yarn ./
    COPY --keep-ts .yarnrc.yml ./

    COPY --keep-ts package.json ./
    COPY --keep-ts acvm-repo/acvm_js/package.json ./acvm-repo/acvm_js/package.json
    COPY --keep-ts compiler/wasm/package.json ./compiler/wasm/package.json
    COPY --keep-ts compiler/integration-tests/package.json ./compiler/integration-tests/package.json
    COPY --keep-ts tooling/noir_codegen/package.json ./tooling/noir_codegen/package.json
    COPY --keep-ts tooling/noir_js/package.json ./tooling/noir_js/package.json
    COPY --keep-ts tooling/noir_js_backend_barretenberg/package.json ./tooling/noir_js_backend_barretenberg/package.json
    COPY --keep-ts tooling/noir_js_types/package.json ./tooling/noir_js_types/package.json
    COPY --keep-ts tooling/noirc_abi_wasm/package.json ./tooling/noirc_abi_wasm/package.json
    COPY --keep-ts docs/package.json ./docs/package.json
    COPY --keep-ts yarn.lock ./

    RUN yarn install --immutable

    # Install other dependencies for building/testing JS packages
    COPY --dir ./.github/scripts /
    RUN /scripts/playwright-install.sh
    RUN /scripts/wasm-bindgen-install.sh
    RUN /scripts/wasm-opt-install.sh
    RUN /scripts/wasm-pack-install.sh

yarn-source:
    FROM +yarn-deps
    COPY --dir --keep-ts compiler/integration-tests compiler/wasm ./compiler
    COPY --dir --keep-ts tooling/noir_codegen tooling/noir_js tooling/noir_js_backend_barretenberg tooling/noir_js_types ./tooling

yarn-build:
    ARG GIT_COMMIT
    FROM +yarn-source
    COPY --dir --keep-ts +source/* .
    
    IF [ -n "${GIT_COMMIT}" ]
        ENV GIT_COMMIT=$GIT_COMMIT
        ENV GIT_DIRTY="false"
    ELSE
        COPY --dir +git-info/* /tmp/git/
        ENV GIT_COMMIT=$(cat /tmp/git/commit_hash)
        ENV GIT_DIRTY=$(cat /tmp/git/dirty)
        RUN rm -rf /tmp/git
    END

    DO rust+SET_CACHE_MOUNTS_ENV
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
        yarn workspaces foreach --parallel --topological-dev --verbose --exclude "docs" --exclude "@noir-lang/root" run build

yarn-lint:
    FROM +yarn-source
    COPY ./.eslintrc.js .
    COPY ./.prettierrc .
    COPY --dir --keep-ts +source/* .
    
    RUN yarn lint

yarn-test:
    FROM +yarn-build
    COPY --dir +build/nargo ./target/release/nargo
    ENV PATH=/project/target/release:$PATH
    
    RUN yarn test

docs-build:
    ARG GIT_COMMIT
    FROM +yarn-source
    COPY --dir --keep-ts +source/* .
    COPY ./docs ./docs

    IF [ -n "${GIT_COMMIT}" ]
        ENV GIT_COMMIT=$GIT_COMMIT
        ENV GIT_DIRTY="false"
    ELSE
        COPY --dir +git-info/* /tmp/git/
        ENV GIT_COMMIT=$(cat /tmp/git/commit_hash)
        ENV GIT_DIRTY=$(cat /tmp/git/dirty)
        RUN rm -rf /tmp/git
    END
    
    DO rust+SET_CACHE_MOUNTS_ENV
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
        yarn workspaces foreach -Rpt --from docs run build
    SAVE ARTIFACT docs/build


all:
  BUILD +fmt
  BUILD +clippy
  BUILD +nargo-fmt
  BUILD +test
  BUILD +yarn-test
