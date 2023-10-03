// Metrics to capture
const L1_ROLLUP_CALLDATA_SIZE_IN_BYTES = "l1_rollup_calldata_size_in_bytes";
const L1_ROLLUP_CALLDATA_GAS = "l1_rollup_calldata_gas";
const L1_ROLLUP_EXECUTION_GAS = "l1_rollup_execution_gas";

// Events to track
const ROLLUP_PUBLISHED_TO_L1 = "rollup-published-to-l1";

// Rollup sizes to track (duplicated from yarn-project/end-to-end/src/benchmarks/bench_publish_rollup.test.ts)
const ROLLUP_SIZES = process.env.ROLLUP_SIZES
  ? process.env.ROLLUP_SIZES.split(",").map(Number)
  : [8, 32, 128];

// Output files
const BENCHMARK_FILE_JSON = process.env.BENCHMARK_FILE_JSON ?? "benchmark.json";

module.exports = {
  L1_ROLLUP_CALLDATA_SIZE_IN_BYTES,
  L1_ROLLUP_CALLDATA_GAS,
  L1_ROLLUP_EXECUTION_GAS,
  ROLLUP_PUBLISHED_TO_L1,
  ROLLUP_SIZES,
  BENCHMARK_FILE_JSON,
};
