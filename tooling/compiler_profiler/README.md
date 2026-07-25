# Noir Compiler Profiler

`noir-compiler-profiler` shows where `nargo compile` itself spends its time. It is an internal
development tool for working on the compiler — to profile Noir *programs* (gates, ACIR opcodes,
Brillig execution) use [`noir-profiler`](../profiler) instead.

## Usage

The easiest entry point is the `just` recipe, run from the repository root:

```bash
just profile-compiler test_programs/benchmarks/semaphore_depth_10
```

This compiles the package with span logging enabled and writes two artifacts to
`target/compiler-profile/`:

- `compiler-flamegraph.svg` — an aggregated, self-time-weighted flamegraph of compiler phases.
  Open it in a browser; frames are clickable.
- `compiler-trace.json` — a per-invocation timeline. Open it in
  [Perfetto](https://ui.perfetto.dev) or `chrome://tracing`.

## How it works

The compiler is instrumented with `tracing` spans (compilation phases, one span per SSA pass,
parsing, ACIR generation, ...). When `nargo` runs with `NARGO_LOG_DIR=<dir> NOIR_LOG=<filter>`
it writes those spans as JSON events to `<dir>`. `noir-compiler-profiler --log-dir <dir>
--output <out>` post-processes the log into the two artifacts:

```bash
log_dir=$(mktemp -d)
NARGO_LOG_DIR="$log_dir" NOIR_LOG="trace,noirc_frontend::elaborator=info" \
    nargo compile --force
noir-compiler-profiler --log-dir "$log_dir" --output profile
```

Use a fresh, empty log directory per run: the log writer appends, so events from a previous run
in the same directory would be merged into the same profile.

## Choosing a span filter

`NOIR_LOG` takes a [`tracing_subscriber::EnvFilter`](https://docs.rs/tracing-subscriber) filter.
All compiler spans are emitted at `trace` level, and two settings are useful:

- `trace,noirc_frontend::elaborator=info` (the recipe default) keeps every compiler phase and
  SSA pass but drops the elaborator's per-expression spans. Logs stay small (megabytes) and the
  timings closely match reality — per-pass durations agree with
  `nargo compile --benchmark-codegen` to the millisecond. Elaboration still shows up, as the
  self time of the `collect_defs_and_elaborate` frame.
- `trace` additionally records the elaborator's per-expression spans, giving a deep breakdown
  of elaboration itself. Expect multi-gigabyte logs and a compile that is an order of magnitude
  slower: with millions of sub-microsecond spans, the logging overhead itself dominates, so
  treat the absolute numbers (and the frontend/backend proportions) as distorted.

## Reading the output

- Frame widths are **busy time** (time the span was entered), aggregated over all instances of
  the same span path. The value unit is microseconds.
- **`(self)` frames** are time spent inside a span but not in any instrumented child — that is,
  un-instrumented work within that phase. A large `(self)` frame is a hint that a phase needs
  more spans, not that the time is unattributable.
- The **`(untracked)` root frame** is wall-clock time not covered by any root span (e.g. process
  startup before `start_cli`).
- Spans do not follow threads. Work spawned on another thread (workspace-wide parallel parsing,
  `compile_program` itself) appears as its own root and is re-parented under the root span whose
  time interval contains it. In such parallel regions the flamegraph sums CPU time across
  threads, so children can add up to more than their parent's wall time (the tool prints a
  warning when they do, and the total can exceed 100% of wall clock).
- The tool prints how much of the wall clock the flamegraph accounts for; with an intact log
  this should be close to 100%. It also warns about spans that never closed, which usually
  means the compile crashed or was killed mid-run.
- The timeline omits spans shorter than `--timeline-min-us` (default 100µs) to keep the JSON
  loadable; pass `--timeline-min-us 0` to keep everything.
