// Given a local folder with the e2e benchmark files, generates a single file
// output with the grouped metrics to be published. This script can probably
// be replaced by a single call to jq, but I found this easier to write,
// and pretty much every CI comes with a working version of node.
//
// To test this locally, first run the benchmark tests from the yarn-project/end-to-end folder
// BENCHMARK=1 yarn test bench
//
// And then run this script from the yarn-project/scripts folder
// LOG_FOLDER=../end-to-end/log yarn bench-aggregate
import {
  type AvmSimulationStats,
  BENCHMARK_BLOCK_SIZES,
  BENCHMARK_HISTORY_BLOCK_SIZE,
  BENCHMARK_HISTORY_CHAIN_LENGTHS,
  type BenchmarkMetricResults,
  type BenchmarkResults,
  type BenchmarkResultsWithTimestamp,
  type CircuitProvingStats,
  type CircuitSimulationStats,
  type CircuitWitnessGenerationStats,
  type L1PublishBlockStats,
  type L2BlockBuiltStats,
  type L2BlockHandledStats,
  type MetricName,
  type NodeSyncedChainHistoryStats,
  type NoteProcessorCaughtUpStats,
  type ProofConstructed,
  type PublicDBAccessStats,
  type Stats,
  type TreeInsertionStats,
  type TxAddedToPoolStats,
} from '@aztec/circuit-types/stats';
import { createConsoleLogger } from '@aztec/foundation/log';

import * as fs from 'fs';
import { mkdirpSync } from 'fs-extra';
import * as path from 'path';
import * as readline from 'readline';

import { BenchDir, BenchFile, LogsDir } from './paths.js';

const log = createConsoleLogger();

/** Appends a data point to the final results for the given metric in the given bucket */
function append(
  results: BenchmarkCollectedResults,
  metric: MetricName,
  bucket: number | string,
  value: number | bigint,
) {
  if (value === undefined) {
    log(`Undefined value for ${metric} in bucket ${bucket}`);
    return;
  }
  const numeric = Number(value);
  if (Number.isNaN(numeric)) {
    log(`Value ${value} for ${metric} in ${bucket} is not a number`);
    return;
  }
  if (!results[metric]) {
    results[metric] = {};
  }
  if (!results[metric]![bucket]) {
    results[metric]![bucket] = [];
  }
  results[metric]![bucket].push(numeric);
}

/** Processes an entry with event name 'acir-proof-generated' and updates results */
function processAcirProofGenerated(entry: ProofConstructed, results: BenchmarkCollectedResults) {
  if (entry.acir_test === 'bench_sha256') {
    append(results, 'proof_construction_time_sha256_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_sha256_30') {
    append(results, 'proof_construction_time_sha256_30_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_sha256_100') {
    append(results, 'proof_construction_time_sha256_100_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_poseidon_hash') {
    append(results, 'proof_construction_time_poseidon_hash_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_poseidon_hash_30') {
    append(results, 'proof_construction_time_poseidon_hash_30_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_poseidon_hash_100') {
    append(results, 'proof_construction_time_poseidon_hash_100_ms', entry.threads, entry.value);
  } else if (entry.acir_test === 'bench_eddsa') {
    append(results, 'proof_construction_time_eddsa_poseidon_ms', entry.threads, entry.value);
  }
}

/** Processes an entry with event name 'rollup-published-to-l1' and updates results */
function processRollupPublished(entry: L1PublishBlockStats, results: BenchmarkCollectedResults) {
  const bucket = entry.txCount;
  if (!BENCHMARK_BLOCK_SIZES.includes(bucket)) {
    return;
  }
  append(results, 'l1_rollup_calldata_gas', bucket, entry.calldataGas);
  append(results, 'l1_rollup_calldata_size_in_bytes', bucket, entry.calldataSize);
  append(results, 'l1_rollup_execution_gas', bucket, entry.gasUsed);
}

/**
 * Processes an entry with event name 'l2-block-handled' and updates results
 * Skips instances where the block was emitted by the same node where the processing is skipped
 */
function processRollupBlockSynced(entry: L2BlockHandledStats, results: BenchmarkCollectedResults) {
  const bucket = entry.txCount;
  if (!BENCHMARK_BLOCK_SIZES.includes(bucket)) {
    return;
  }
  if (entry.isBlockOurs) {
    return;
  }
  append(results, 'l2_block_processing_time_in_ms', bucket, entry.duration);
}

/**
 * Processes an entry with event name 'circuit-simulated' and updates results
 * Buckets are circuit names
 */
function processCircuitSimulation(entry: CircuitSimulationStats, results: BenchmarkCollectedResults) {
  if (entry.circuitName === 'app-circuit') {
    // app circuits aren't simulated
    return;
  } else {
    const bucket = entry.circuitName;
    append(results, 'protocol_circuit_simulation_time_in_ms', bucket, entry.duration);
    append(results, 'protocol_circuit_input_size_in_bytes', bucket, entry.inputSize);
    append(results, 'protocol_circuit_output_size_in_bytes', bucket, entry.outputSize);
  }
}

/**
 * Processes an entry with event name 'circuit-proving' and updates results
 * Buckets are circuit names
 */
function processCircuitProving(entry: CircuitProvingStats, results: BenchmarkCollectedResults) {
  if (entry.circuitName === 'app-circuit') {
    if (!entry.appCircuitName) {
      return;
    }
    const bucket = entry.appCircuitName;
    append(results, 'app_circuit_proving_time_in_ms', bucket, entry.duration);
    append(results, 'app_circuit_proof_size_in_bytes', bucket, entry.proofSize);
    append(results, 'app_circuit_size_in_gates', bucket, entry.circuitSize);
    append(results, 'app_circuit_num_public_inputs', bucket, entry.numPublicInputs);
  } else if (entry.circuitName === 'avm-circuit') {
    if (!entry.appCircuitName) {
      return;
    }
    const bucket = `${entry.appCircuitName} (avm)`;
    append(results, 'app_circuit_proving_time_in_ms', bucket, entry.duration);
    append(results, 'app_circuit_proof_size_in_bytes', bucket, entry.proofSize);
    append(results, 'app_circuit_input_size_in_bytes', bucket, entry.inputSize);
    // These are not yet correctly passed in bb_prover.ts.
    // append(results, 'app_circuit_size_in_gates', bucket, entry.circuitSize);
    // append(results, 'app_circuit_num_public_inputs', bucket, entry.numPublicInputs);
  } else {
    const bucket = entry.circuitName;
    append(results, 'protocol_circuit_proving_time_in_ms', bucket, entry.duration);
    append(results, 'protocol_circuit_proof_size_in_bytes', bucket, entry.proofSize);
    append(results, 'protocol_circuit_size_in_gates', bucket, entry.circuitSize);
    append(results, 'protocol_circuit_num_public_inputs', bucket, entry.numPublicInputs);
  }
}

function processAvmSimulation(entry: AvmSimulationStats, results: BenchmarkCollectedResults) {
  append(results, 'avm_simulation_time_ms', entry.appCircuitName, entry.duration);
  append(results, 'avm_simulation_bytecode_size_in_bytes', entry.appCircuitName, entry.bytecodeSize);
}

function processDbAccess(entry: PublicDBAccessStats, results: BenchmarkCollectedResults) {
  append(results, 'public_db_access_time_ms', entry.operation, entry.duration);
}

/**
 * Processes an entry with event name 'circuit-proving' and updates results
 * Buckets are circuit names
 */
function processCircuitWitnessGeneration(entry: CircuitWitnessGenerationStats, results: BenchmarkCollectedResults) {
  if (entry.circuitName === 'app-circuit') {
    const bucket = entry.appCircuitName;
    if (!bucket) {
      return;
    }
    append(results, 'app_circuit_witness_generation_time_in_ms', bucket, entry.duration);
    append(results, 'app_circuit_input_size_in_bytes', bucket, entry.inputSize);
    append(results, 'app_circuit_output_size_in_bytes', bucket, entry.outputSize);
  } else {
    const bucket = entry.circuitName;
    append(results, 'protocol_circuit_witness_generation_time_in_ms', bucket, entry.duration);
  }
}
/**
 * Processes an entry with event name 'note-processor-caught-up' and updates results
 */
function processNoteProcessorCaughtUp(entry: NoteProcessorCaughtUpStats, results: BenchmarkCollectedResults) {
  const { decryptedIncoming, decryptedOutgoing, blocks, dbSize } = entry;
  if (BENCHMARK_HISTORY_CHAIN_LENGTHS.includes(blocks) && (decryptedIncoming > 0 || decryptedOutgoing > 0)) {
    append(results, 'pxe_database_size_in_bytes', blocks, dbSize);
  }
}

/** Processes an entry with event name 'l2-block-built' and updates results where buckets are rollup sizes */
function processL2BlockBuilt(entry: L2BlockBuiltStats, results: BenchmarkCollectedResults) {
  const bucket = entry.txCount;
  if (!BENCHMARK_BLOCK_SIZES.includes(bucket)) {
    return;
  }
  append(results, 'l2_block_building_time_in_ms', bucket, entry.duration);
  append(results, 'l2_block_rollup_simulation_time_in_ms', bucket, entry.rollupCircuitsDuration);
  append(results, 'l2_block_public_tx_process_time_in_ms', bucket, entry.publicProcessDuration);
}

/** Processes entries with event name node-synced-chain-history emitted by benchmark tests where buckets are chain lengths */
function processNodeSyncedChain(entry: NodeSyncedChainHistoryStats, results: BenchmarkCollectedResults) {
  const bucket = entry.blockCount;
  if (!BENCHMARK_HISTORY_CHAIN_LENGTHS.includes(bucket)) {
    return;
  }
  if (entry.txsPerBlock !== BENCHMARK_HISTORY_BLOCK_SIZE) {
    return;
  }
  append(results, 'node_history_sync_time_in_ms', bucket, entry.duration);
  append(results, 'node_database_size_in_bytes', bucket, entry.dbSize);
}

/** Processes entries for events tx-added-to-pool, with grouping by deployed contract count. */
function processTxAddedToPool(entry: TxAddedToPoolStats, results: BenchmarkCollectedResults) {
  append(results, 'tx_size_in_bytes', entry.classRegisteredCount, entry.size);
}

/** Process a tree insertion event and updates results */
function processTreeInsertion(entry: TreeInsertionStats, results: BenchmarkCollectedResults) {
  const bucket = entry.batchSize;
  const depth = entry.treeDepth;
  if (entry.treeType === 'append-only') {
    if (depth === 16) {
      append(results, 'batch_insert_into_append_only_tree_16_depth_ms', bucket, entry.duration);
      append(results, 'batch_insert_into_append_only_tree_16_depth_hash_count', bucket, entry.hashCount);
      append(results, 'batch_insert_into_append_only_tree_16_depth_hash_ms', bucket, entry.hashDuration);
    } else if (depth === 32) {
      append(results, 'batch_insert_into_append_only_tree_32_depth_ms', bucket, entry.duration);
      append(results, 'batch_insert_into_append_only_tree_32_depth_hash_count', bucket, entry.hashCount);
      append(results, 'batch_insert_into_append_only_tree_32_depth_hash_ms', bucket, entry.hashDuration);
    }
  } else if (entry.treeType === 'indexed') {
    if (depth === 20) {
      append(results, 'batch_insert_into_indexed_tree_20_depth_ms', bucket, entry.duration);
      append(results, 'batch_insert_into_indexed_tree_20_depth_hash_count', bucket, entry.hashCount);
      append(results, 'batch_insert_into_indexed_tree_20_depth_hash_ms', bucket, entry.hashDuration);
    } else if (depth === 40) {
      append(results, 'batch_insert_into_indexed_tree_40_depth_ms', bucket, entry.duration);
      append(results, 'batch_insert_into_indexed_tree_40_depth_hash_count', bucket, entry.hashCount);
      append(results, 'batch_insert_into_indexed_tree_40_depth_hash_ms', bucket, entry.hashDuration);
    }
  }
}

/** Processes a parsed entry from a log-file and updates results */
function processEntry(entry: Stats, results: BenchmarkCollectedResults) {
  switch (entry.eventName) {
    case 'proof_construction_time':
      return processAcirProofGenerated(entry, results);
    case 'rollup-published-to-l1':
      return processRollupPublished(entry, results);
    case 'l2-block-handled':
      return processRollupBlockSynced(entry, results);
    case 'circuit-simulation':
      return processCircuitSimulation(entry, results);
    case 'circuit-witness-generation':
      return processCircuitWitnessGeneration(entry, results);
    case 'circuit-proving':
      return processCircuitProving(entry, results);
    case 'note-processor-caught-up':
      return processNoteProcessorCaughtUp(entry, results);
    case 'l2-block-built':
      return processL2BlockBuilt(entry, results);
    case 'node-synced-chain-history':
      return processNodeSyncedChain(entry, results);
    case 'tx-added-to-pool':
      return processTxAddedToPool(entry, results);
    case 'tree-insertion':
      return processTreeInsertion(entry, results);
    case 'avm-simulation':
      return processAvmSimulation(entry, results);
    case 'public-db-access':
      return processDbAccess(entry, results);
    default:
      return;
  }
}

/** Array of collected raw results for a given metric. */
type BenchmarkCollectedMetricResults = Record<string, number[]>;

/** Collected raw results pending averaging each bucket within each metric. */
type BenchmarkCollectedResults = Partial<Record<MetricName, BenchmarkCollectedMetricResults>>;

/** Parses all jsonl files downloaded and aggregates them into a single results object. */
export async function main() {
  const collected: BenchmarkCollectedResults = {};

  // Get all jsonl files in the logs dir
  const files = fs.readdirSync(LogsDir).filter(f => f.endsWith('.jsonl'));

  // Iterate over each .jsonl file
  for (const file of files) {
    const filePath = path.join(LogsDir, file);
    const fileStream = fs.createReadStream(filePath);
    const rl = readline.createInterface({ input: fileStream });

    for await (const line of rl) {
      const entry = JSON.parse(line);
      processEntry(entry, collected);
    }
  }

  log(`Collected entries: ${JSON.stringify(collected)}`);

  // For each bucket of each metric compute the average all collected data points
  const results: BenchmarkResults = {};
  for (const [metricName, metric] of Object.entries(collected)) {
    const resultMetric: BenchmarkMetricResults = {};
    results[metricName as MetricName] = resultMetric;
    for (const [bucketName, bucket] of Object.entries(metric)) {
      let avg = bucket.reduce((acc, val) => acc + val, 0) / bucket.length;
      if (avg > 100) {
        avg = Math.floor(avg);
      }
      resultMetric[bucketName] = avg;
    }
  }

  const timestampedResults: BenchmarkResultsWithTimestamp = { ...results, timestamp: new Date().toISOString() };

  // Write results to disk
  log(`Aggregated results: ${JSON.stringify(timestampedResults, null, 2)}`);
  mkdirpSync(BenchDir);
  fs.writeFileSync(BenchFile, JSON.stringify(timestampedResults, null, 2));
}
