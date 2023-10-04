// Rollup sizes to track (duplicated from yarn-project/end-to-end/src/benchmarks/bench_publish_rollup.test.ts)
const ROLLUP_SIZES = process.env.ROLLUP_SIZES
  ? process.env.ROLLUP_SIZES.split(",").map(Number)
  : [8, 32, 128];

// Output files
const BENCHMARK_FILE_JSON = process.env.BENCHMARK_FILE_JSON ?? "benchmark.json";

module.exports = {
  // Metrics to capture
  L1_ROLLUP_CALLDATA_SIZE_IN_BYTES: "l1_rollup_calldata_size_in_bytes",
  L1_ROLLUP_CALLDATA_GAS: "l1_rollup_calldata_gas",
  L1_ROLLUP_EXECUTION_GAS: "l1_rollup_execution_gas",
  L2_BLOCK_PROCESSING_TIME: "l2_block_processing_time_in_ms",
  CIRCUIT_SIMULATION_TIME: "circuit_simulation_time_in_ms",
  CIRCUIT_INPUT_SIZE: "circuit_input_size_in_bytes",
  CIRCUIT_OUTPUT_SIZE: "circuit_output_size_in_bytes",
  // Events to track
  L2_BLOCK_PUBLISHED_TO_L1: "rollup-published-to-l1",
  L2_BLOCK_SYNCED: "l2-block-handled",
  CIRCUIT_SIMULATED: "circuit-simulation",
  // Other
  ROLLUP_SIZES,
  BENCHMARK_FILE_JSON,
};
