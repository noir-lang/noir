# ContFuzzer-compatible container for the AST fuzzer.
#
# Builds all 6 libFuzzer targets that differentially test the Noir AST
# compilation pipeline. Each target compares two execution paths — when
# they disagree, the binary panics and libFuzzer catches it as a crash.
#
# Targets:
#   acir_vs_brillig, min_vs_full, pass_vs_prev, orig_vs_morph,
#   comptime_vs_brillig_nargo, comptime_vs_brillig_direct
#
# Build:
#   docker build -t contfuzzer-ast-fuzzer -f tooling/ast_fuzzer/contfuzzer-container/Dockerfile .
#
# Run standalone (platform-like flags):
#   docker run --read-only --tmpfs /tmp:size=512m \
#       --user 65534:10001 \
#       -v /tmp/corpus:/corpus -v /tmp/crashes:/crashes -v /tmp/output:/output \
#       -e FUZZ_MODE=fuzz -e FUZZ_TARGET=acir_vs_brillig -e FUZZ_TIMEOUT=60 \
#       contfuzzer-ast-fuzzer

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

WORKDIR /noir/tooling/ast_fuzzer

# Build all 6 fuzz targets (ASan + libFuzzer instrumentation).
RUN cargo +nightly fuzz build acir_vs_brillig --fuzz-dir ./fuzz
RUN cargo +nightly fuzz build min_vs_full --fuzz-dir ./fuzz
RUN cargo +nightly fuzz build pass_vs_prev --fuzz-dir ./fuzz
RUN cargo +nightly fuzz build orig_vs_morph --fuzz-dir ./fuzz
RUN cargo +nightly fuzz build comptime_vs_brillig_nargo --fuzz-dir ./fuzz
RUN cargo +nightly fuzz build comptime_vs_brillig_direct --fuzz-dir ./fuzz

# Build coverage-instrumented variants for FUZZ_MODE=coverage.
# Uses -Cinstrument-coverage instead of ASan — produces binaries that emit
# .profraw files for llvm-cov when LLVM_PROFILE_FILE is set.
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage acir_vs_brillig --fuzz-dir ./fuzz -- /tmp/empty-corpus || true
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage min_vs_full --fuzz-dir ./fuzz -- /tmp/empty-corpus || true
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage pass_vs_prev --fuzz-dir ./fuzz -- /tmp/empty-corpus || true
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage orig_vs_morph --fuzz-dir ./fuzz -- /tmp/empty-corpus || true
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage comptime_vs_brillig_nargo --fuzz-dir ./fuzz -- /tmp/empty-corpus || true
RUN mkdir -p /tmp/empty-corpus && \
    cargo +nightly fuzz coverage comptime_vs_brillig_direct --fuzz-dir ./fuzz -- /tmp/empty-corpus || true

# Copy both variants into /targets/{fuzzing,coverage}/.
# Fuzz binaries: target/<triple>/release/<target>
# Coverage binaries: fuzz/coverage/<triple>/release/<target>
RUN mkdir -p /targets/fuzzing /targets/coverage && \
    for target in acir_vs_brillig min_vs_full pass_vs_prev orig_vs_morph comptime_vs_brillig_nargo comptime_vs_brillig_direct; do \
        cp /noir/target/*/release/$target /targets/fuzzing/ && \
        find /noir/tooling/ast_fuzzer/fuzz -path "*/coverage/*/release/$target" \
            -exec cp {} /targets/coverage/ \; && \
        test -x /targets/fuzzing/$target || { echo "FATAL: $target fuzzing binary missing"; exit 1; } && \
        test -x /targets/coverage/$target || echo "WARN: $target coverage binary missing — coverage mode will be unavailable"; \
    done

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
COPY --from=builder /noir/tooling/ast_fuzzer/contfuzzer-container/entrypoint.sh /entrypoint.sh
COPY --from=builder /noir/tooling/ast_fuzzer/contfuzzer-container/fuzzer_manifest.json /fuzzer_manifest.json
RUN chmod +x /entrypoint.sh

# World-readable — platform runs as UID 65534 (nobody).
RUN chmod -R a+rX /targets /noir

WORKDIR /noir/tooling/ast_fuzzer

ENTRYPOINT ["/entrypoint.sh"]
