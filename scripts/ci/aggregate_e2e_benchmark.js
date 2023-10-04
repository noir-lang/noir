// Given a local folder with the e2e benchmark files, generates a single file
// output with the grouped metrics to be published. This script can probably
// be replaced by a single call to jq, but I found this easier to write,
// and pretty much every CI comes with a working version of node.
//
// To test this locally, first run the benchmark tests from the yarn-project/end-to-end folder
// BENCHMARK=1 ROLLUP_SIZES=8 yarn test bench
//
// And then run this script from the root of the project:
// LOGS_DIR=./yarn-project/end-to-end/log/ node ./scripts/ci/aggregate_e2e_benchmark.js 

const fs = require("fs");
const path = require("path");
const readline = require("readline");

const {
  L1_ROLLUP_CALLDATA_SIZE_IN_BYTES,
  L1_ROLLUP_CALLDATA_GAS,
  L1_ROLLUP_EXECUTION_GAS,
  L2_BLOCK_PROCESSING_TIME,
  L2_BLOCK_SYNCED,
  L2_BLOCK_PUBLISHED_TO_L1,
  CIRCUIT_SIMULATION_TIME,
  CIRCUIT_OUTPUT_SIZE,
  CIRCUIT_INPUT_SIZE,
  CIRCUIT_SIMULATED,
  ROLLUP_SIZES,
  BENCHMARK_FILE_JSON,
} = require("./benchmark_shared.js");

// Folder where to load logs from
const logsDir = process.env.LOGS_DIR ?? `log`;

// Appends a datapoint to the final results for the given metric in the given bucket
function append(results, metric, bucket, value) {
  if (value === undefined) {
    console.error(`Undefined value for ${metric} in bucket ${bucket}`);
    return;
  }
  const numeric = Number(value);
  if (Number.isNaN(numeric)) {
    console.error(`Value ${value} for ${metric} in ${bucket} is not a number`);
    return;
  }
  if (!results[metric]) results[metric] = {};
  if (!results[metric][bucket]) results[metric][bucket] = [];
  results[metric][bucket].push(numeric);
}

// Processes an entry with event name 'rollup-published-to-l1' and updates results
function processRollupPublished(entry, results) {
  const bucket = entry.txCount;
  if (!ROLLUP_SIZES.includes(bucket)) return;
  append(results, L1_ROLLUP_CALLDATA_GAS, bucket, entry.calldataGas);
  append(results, L1_ROLLUP_CALLDATA_SIZE_IN_BYTES, bucket, entry.calldataSize);
  append(results, L1_ROLLUP_EXECUTION_GAS, bucket, entry.gasUsed);
}

// Processes an entry with event name 'l2-block-handled' and updates results
// Skips instances where the block was emitted by the same node where the processing is skipped
function processRollupBlockSynced(entry, results) {
  const bucket = entry.txCount;
  if (!ROLLUP_SIZES.includes(bucket)) return;
  if (entry.isBlockOurs) return;
  append(results, L2_BLOCK_PROCESSING_TIME, bucket, entry.duration);
}

// Processes an entry with event name 'circuit-simulated' and updates results
// Buckets are circuit names
function processCircuitSimulation(entry, results) {
  const bucket = entry.circuitName;
  if (!bucket) return;
  append(results, CIRCUIT_SIMULATION_TIME, bucket, entry.duration);
  append(results, CIRCUIT_INPUT_SIZE, bucket, entry.inputSize);
  append(results, CIRCUIT_OUTPUT_SIZE, bucket, entry.outputSize);
}

// Processes a parsed entry from a logfile and updates results
function processEntry(entry, results) {
  switch (entry.eventName) {
    case L2_BLOCK_PUBLISHED_TO_L1:
      return processRollupPublished(entry, results);
    case L2_BLOCK_SYNCED:
      return processRollupBlockSynced(entry, results);
    case CIRCUIT_SIMULATED:
      return processCircuitSimulation(entry, results);
    default:
      return;
  }
}

// Parses all jsonl files downloaded and aggregates them into a single results object
async function main() {
  const results = {};

  // Get all jsonl files in the logs dir
  const files = fs.readdirSync(logsDir).filter((f) => f.endsWith(".jsonl"));

  // Iterate over each .jsonl file
  for (const file of files) {
    const filePath = path.join(logsDir, file);
    const fileStream = fs.createReadStream(filePath);
    const rl = readline.createInterface({ input: fileStream });

    for await (const line of rl) {
      const entry = JSON.parse(line);
      processEntry(entry, results);
    }
  }

  console.log(`Collected entries:`, JSON.stringify(results, null, 2));

  // For each bucket of each metric compute the average all collected datapoints
  for (const metricName in results) {
    const metric = results[metricName];
    for (const bucketName in metric) {
      const bucket = metric[bucketName];
      let avg = bucket.reduce((acc, val) => acc + val, 0) / bucket.length;
      if (avg > 100) avg = Math.floor(avg);
      metric[bucketName] = avg;
    }
  }

  // Throw in a timestamp
  results.timestamp = new Date().toISOString();

  // Write results to disk
  console.log(`Aggregated results:`, JSON.stringify(results, null, 2));
  fs.writeFileSync(BENCHMARK_FILE_JSON, JSON.stringify(results, null, 2));
}

main();
