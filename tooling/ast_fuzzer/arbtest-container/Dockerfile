# ContFuzzer-compatible container for the AST fuzzer (arbtest variant).
#
# Pre-builds the test binary during docker build, then runs it directly
# at runtime — no cargo needed. This avoids cargo fingerprint invalidation,
# network access requirements, and git safe.directory issues.
#
# Build:
#   docker build -t contfuzzer-ast-arbtest -f tooling/ast_fuzzer/arbtest-container/Dockerfile .
#
# Build from a specific noir commit:
#   docker build --build-arg COMMIT=abc123 -t contfuzzer-ast-arbtest .
#
# Run standalone (platform-like flags):
#   docker run --read-only --tmpfs /tmp:size=512m \
#       --user 65534:10001 \
#       -v /tmp/crashes:/crashes -v /tmp/output:/output \
#       -e FUZZ_MODE=fuzz -e FUZZ_TARGET=comptime_vs_brillig_direct -e FUZZ_TIMEOUT=300 \
#       contfuzzer-ast-arbtest

# ── Builder ────────────────────────────────────────────────────────────
FROM --platform=linux/amd64 rust:1.89.0-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        git \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Source the noir workspace. Resolution order:
#   COMMIT set:   clone from GitHub and checkout that commit (wins over BRANCH/CI).
#   BRANCH set:   shallow-clone that branch from GitHub.
#   CI_BUILD set: use the already checked-out repo COPY'd into the context.
#   default:      shallow-clone master from GitHub.
ARG COMMIT=""
ARG BRANCH=""
ARG CI_BUILD=""
COPY . /noir-context
RUN if [ -n "$COMMIT" ]; then \
        rm -rf /noir-context && \
        git clone https://github.com/noir-lang/noir.git --depth 1 /noir && \
        cd /noir && git fetch origin "$COMMIT" --depth 1 && git checkout "$COMMIT"; \
    elif [ -n "$BRANCH" ]; then \
        rm -rf /noir-context && \
        git clone --depth 1 --branch "$BRANCH" https://github.com/noir-lang/noir.git /noir; \
    elif [ -n "$CI_BUILD" ]; then \
        mv /noir-context /noir; \
    else \
        rm -rf /noir-context && \
        git clone https://github.com/noir-lang/noir.git --depth 1 /noir; \
    fi

WORKDIR /noir

# Build the test binary. cargo test --no-run compiles but doesn't execute.
# The binary lands at target/release/deps/noir_ast_fuzzer_fuzz-<hash>.
RUN cargo test --release -p noir_ast_fuzzer_fuzz --no-run

# Find the test binary and copy it to a known location.
# There's exactly one executable matching this pattern.
# Copy the test binary and create per-target symlinks so the platform's
# image scanner finds an entry in /targets/ for every manifest target.
# All 6 targets live inside the same binary — the entrypoint passes the
# target name as a substring filter at runtime.
RUN mkdir -p /targets/fuzzing && \
    TEST_BIN=$(find target/release/deps -name 'noir_ast_fuzzer_fuzz-*' -type f -executable | head -1) && \
    test -n "$TEST_BIN" || { echo "FATAL: test binary not found"; exit 1; } && \
    cp "$TEST_BIN" /targets/fuzzing/noir_ast_fuzzer_fuzz && \
    for t in acir_vs_brillig min_vs_full pass_vs_prev orig_vs_morph \
             comptime_vs_brillig_nargo comptime_vs_brillig_direct; do \
        printf '#!/bin/sh\nexec /targets/fuzzing/noir_ast_fuzzer_fuzz "$@"\n' \
            > "/targets/fuzzing/$t" && chmod +x "/targets/fuzzing/$t"; \
    done

# ── Runtime ────────────────────────────────────────────────────────────
FROM --platform=linux/amd64 ubuntu:24.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /targets /targets

COPY tooling/ast_fuzzer/arbtest-container/entrypoint.sh /entrypoint.sh
COPY tooling/ast_fuzzer/arbtest-container/fuzzer_manifest.json /fuzzer_manifest.json
RUN chmod +x /entrypoint.sh

# World-readable — platform runs as UID 65534 (nobody).
RUN chmod -R a+rX /targets

ENTRYPOINT ["/entrypoint.sh"]
