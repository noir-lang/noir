// Metrics to capture
const L1_ROLLUP_CALLDATA_SIZE_IN_BYTES = "l1_rollup_calldata_size_in_bytes";
const L1_ROLLUP_CALLDATA_GAS = "l1_rollup_calldata_gas";
const L1_ROLLUP_EXECUTION_GAS = "l1_rollup_execution_gas";
const L2_BLOCK_PROCESSING_TIME = "l2_block_processing_time_in_ms";

// Events to track
const L2_BLOCK_PUBLISHED_TO_L1 = "rollup-published-to-l1";
const L2_BLOCK_SYNCED = "l2-block-handled";

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
  L2_BLOCK_PROCESSING_TIME,
  L2_BLOCK_PUBLISHED_TO_L1,
  L2_BLOCK_SYNCED,
  ROLLUP_SIZES,
  BENCHMARK_FILE_JSON,
};
