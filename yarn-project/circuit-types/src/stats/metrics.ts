import { StatsEventName } from './stats.js';

/** How a metric is grouped in benchmarks: by block size, by length of chain processed, or by circuit name. */
export type MetricGroupBy =
  | 'block-size'
  | 'chain-length'
  | 'circuit-name'
  | 'contract-count'
  | 'leaf-count'
  | 'data-writes';

/** Definition of a metric to track in benchmarks. */
export interface Metric {
  /** Identifier. */
  name: string;
  /** What dimension this metric is grouped by. */
  groupBy: MetricGroupBy;
  /** Description */
  description: string;
  /** Events used for generating this metric. */
  events: readonly StatsEventName[];
}

/** Metric definitions to track from benchmarks. */
export const Metrics = [
  {
    name: 'l1_rollup_calldata_size_in_bytes',
    groupBy: 'block-size',
    description: 'Size in bytes of the tx calldata posted to L1 when submitting a block.',
    events: ['rollup-published-to-l1'],
  },
  {
    name: 'l1_rollup_calldata_gas',
    groupBy: 'block-size',
    description: 'Estimated gas cost of the tx calldata when posting a block to L1.',
    events: ['rollup-published-to-l1'],
  },
  {
    name: 'l1_rollup_execution_gas',
    groupBy: 'block-size',
    description: 'Total gas used in a tx when submitting a block to L1.',
    events: ['rollup-published-to-l1'],
  },
  {
    name: 'l2_block_processing_time_in_ms',
    groupBy: 'block-size',
    description: 'Time for the state synchronizer to process an L2 block that was not built by its own sequencer.',
    events: ['l2-block-handled'],
  },
  {
    name: 'note_successful_decrypting_time_in_ms',
    groupBy: 'block-size',
    description: 'Time for the PXE to decrypt all notes in a block where they belong to a registered account.',
    events: ['note-processor-caught-up'],
  },
  {
    name: 'note_trial_decrypting_time_in_ms',
    groupBy: 'block-size',
    description:
      'Time for the PXE to try decrypting all notes in a block where they do not belong to a registered account.',
    events: ['note-processor-caught-up'],
  },
  {
    name: 'l2_block_building_time_in_ms',
    groupBy: 'block-size',
    description: 'Total time for the sequencer to build an L2 block from a set of txs.',
    events: ['l2-block-built'],
  },
  {
    name: 'l2_block_rollup_simulation_time_in_ms',
    groupBy: 'block-size',
    description: 'Time for the sequencer to run the rollup circuit simulation when assembling a block.',
    events: ['l2-block-built'],
  },
  {
    name: 'l2_block_public_tx_process_time_in_ms',
    groupBy: 'block-size',
    description: 'Time for the sequencer to execute public function calls for txs in a block.',
    events: ['l2-block-built'],
  },
  {
    name: 'node_history_sync_time_in_ms',
    groupBy: 'chain-length',
    description: 'Time for a node without a sequencer to sync chain history',
    events: ['node-synced-chain-history'],
  },
  {
    name: 'note_history_successful_decrypting_time_in_ms',
    groupBy: 'chain-length',
    description: 'Time for a PXE to decrypt all notes in the chain where they belong to a registered account.',
    events: ['note-processor-caught-up'],
  },
  {
    name: 'note_history_trial_decrypting_time_in_ms',
    groupBy: 'chain-length',
    description:
      'Time for a PXE to try decrypting all notes in the chain where they do not belong to a registered account.',
    events: ['note-processor-caught-up'],
  },
  {
    name: 'node_database_size_in_bytes',
    groupBy: 'chain-length',
    description: 'Size on disk of the leveldown database of a node after syncing all chain history.',
    events: ['node-synced-chain-history'],
  },
  {
    name: 'pxe_database_size_in_bytes',
    groupBy: 'chain-length',
    description: 'Estimated size in memory of a PXE database after syncing all notes that belong to it in the chain.',
    events: ['note-processor-caught-up'],
  },
  {
    name: 'circuit_simulation_time_in_ms',
    groupBy: 'circuit-name',
    description: 'Time to run a circuit simulation.',
    events: ['circuit-simulation'],
  },
  {
    name: 'circuit_input_size_in_bytes',
    groupBy: 'circuit-name',
    description: 'Size of the inputs to a circuit simulation.',
    events: ['circuit-simulation'],
  },
  {
    name: 'circuit_output_size_in_bytes',
    groupBy: 'circuit-name',
    description: 'Size of the outputs (ie public inputs) from a circuit simulation.',
    events: ['circuit-simulation'],
  },
  {
    name: 'tx_size_in_bytes',
    groupBy: 'contract-count',
    description: 'Size of txs received in the mempool.',
    events: ['tx-added-to-pool'],
  },
  {
    name: 'tx_pxe_processing_time_ms',
    groupBy: 'data-writes',
    description: 'Time to process the private part of a tx.',
    events: ['tx-pxe-processing'],
  },
  {
    name: 'tx_sequencer_processing_time_ms',
    groupBy: 'data-writes',
    description: 'Time to process the public part of a tx.',
    events: ['tx-sequencer-processing'],
  },
  {
    name: 'batch_insert_into_append_only_tree_16_depth_ms',
    groupBy: 'leaf-count',
    description: 'Time to insert a batch of leaves into an append-only tree',
    events: ['tree-insertion'],
  },
  {
    name: 'batch_insert_into_append_only_tree_32_depth_ms',
    groupBy: 'leaf-count',
    description: 'Time to insert a batch of leaves into an append-only tree',
    events: ['tree-insertion'],
  },
  {
    name: 'batch_insert_into_indexed_tree_20_depth_ms',
    groupBy: 'leaf-count',
    description: 'Time to insert a batch of leaves into an indexed tree',
    events: ['tree-insertion'],
  },
  {
    name: 'batch_insert_into_indexed_tree_40_depth_ms',
    groupBy: 'leaf-count',
    description: 'Time to insert a batch of leaves into an indexed tree',
    events: ['tree-insertion'],
  },
] as const satisfies readonly Metric[];

/** Metric definitions to track from benchmarks. */
export type Metrics = typeof Metrics;

/** Type of valid metric names. */
export type MetricName = Metrics[number]['name'];
