# ContFuzzer-compatible container for the SSA fuzzer (acir_vs_brillig target).
#
# Builds the libFuzzer binary that differentially tests ACIR vs Brillig
# execution of randomly generated SSA programs. When the two runtimes
# disagree, the binary panics — libFuzzer catches it as a crash.
#
# Build:
#   docker build -t contfuzzer-ssa-fuzzer .
#
# Build from a specific noir commit:
#   docker build --build-arg COMMIT=abc123 -t contfuzzer-ssa-fuzzer .
#
# Run standalone (platform-like flags):
#   docker run --read-only --tmpfs /tmp:size=512m \
#       --user 65534:10001 \
#       -v /tmp/corpus:/corpus -v /tmp/crashes:/crashes -v /tmp/output:/output \
#       -e FUZZ_MODE=fuzz -e FUZZ_TARGET=acir_vs_brillig -e FUZZ_TIMEOUT=60 \
#       contfuzzer-ssa-fuzzer

# ── Builder ────────────────────────────────────────────────────────────
FROM --platform=linux/amd64 rustlang/rust:nightly-bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential cmake clang llvm-dev libclang-dev git \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-fuzz

# Source the noir workspace. Resolution order:
#   COMMIT set: clone from GitHub and checkout that commit (wins over BRANCH/CI).
#   BRANCH set: shallow-clone that branch from GitHub.
#   CI set:     use the already checked-out repo COPY'd into the context.
#   default:    shallow-clone master from GitHub.
ARG COMMIT=""
ARG BRANCH=""
ARG CI=""
COPY . /noir-context
RUN if [ -n "$COMMIT" ]; then \
        rm -rf /noir-context && \
        git clone https://github.com/noir-lang/noir.git --depth 1 /noir && \
        cd /noir && git fetch origin "$COMMIT" --depth 1 && git checkout "$COMMIT"; \
    elif [ -n "$BRANCH" ]; then \
        rm -rf /noir-context && \
        git clone --depth 1 --branch "$BRANCH" https://github.com/noir-lang/noir.git /noir; \
    elif [ -n "$CI" ]; then \
        mv /noir-context /noir; \
    else \
        rm -rf /noir-context && \
        git clone https://github.com/noir-lang/noir.git --depth 1 /noir; \
    fi

WORKDIR /noir/tooling/ssa_fuzzer

# Build the acir_vs_brillig fuzz target (ASan + libFuzzer instrumentation).
RUN cargo +nightly fuzz build acir_vs_brillig --fuzz-dir ./fuzzer

# Build a coverage-instrumented variant for FUZZ_MODE=coverage.
# Uses -Cinstrument-coverage instead of ASan — produces a binary that emits
# .profraw files for llvm-cov when LLVM_PROFILE_FILE is set.
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage acir_vs_brillig --fuzz-dir ./fuzzer -- /tmp/empty-corpus || true

# Copy both variants into /targets/{fuzzing,coverage}/.
# Fuzz binary: target/<triple>/release/acir_vs_brillig
# Coverage binary: fuzzer/coverage/<triple>/release/acir_vs_brillig
RUN mkdir -p /targets/fuzzing /targets/coverage && \
    cp /noir/target/*/release/acir_vs_brillig /targets/fuzzing/ && \
    find /noir/tooling/ssa_fuzzer/fuzzer -path "*/coverage/*/release/acir_vs_brillig" \
        -exec cp {} /targets/coverage/ \; && \
    test -x /targets/fuzzing/acir_vs_brillig || { echo "FATAL: fuzzing binary missing"; exit 1; } && \
    test -x /targets/coverage/acir_vs_brillig || echo "WARN: coverage binary missing — coverage mode will be unavailable"

# ── Runtime ────────────────────────────────────────────────────────────
FROM --platform=linux/amd64 ubuntu:24.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libstdc++6 \
        llvm-18 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /targets /targets

# Source tree for llvm-cov HTML reports — path must match the builder's /noir
# so llvm-cov can resolve source locations embedded in the coverage binary.
COPY --from=builder /noir /noir

# Use the standard ContFuzzer libFuzzer adapter entrypoint.
# It translates FUZZ_* env vars into libFuzzer CLI arguments.
# Copy entrypoint from the source tree (always at /noir after builder stage).
COPY --from=builder /noir/tooling/ssa_fuzzer/contfuzzer-container/entrypoint.sh /entrypoint.sh
COPY --from=builder /noir/tooling/ssa_fuzzer/contfuzzer-container/fuzzer_manifest.json /fuzzer_manifest.json
RUN chmod +x /entrypoint.sh

# World-readable — platform runs as UID 65534 (nobody).
RUN chmod -R a+rX /targets /noir

ENTRYPOINT ["/entrypoint.sh"]
