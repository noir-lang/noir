/**
 * @file Metric names used in Aztec.
 * Metric names must be unique and not clash with {@link attributes.ts | Attribute names}.
 * Prefix metric names with `aztec` and use dots `.` to separate namespaces.
 *
 * @see {@link https://opentelemetry.io/docs/specs/semconv/general/metrics/ | OpenTelemetry Metrics} for naming conventions.
 */

/** How long it takes to simulate a circuit */
export const CIRCUIT_SIMULATION_DURATION = 'aztec.circuit.simulation.duration';
export const CIRCUIT_SIMULATION_INPUT_SIZE = 'aztec.circuit.simulation.input_size';
export const CIRCUIT_SIMULATION_OUTPUT_SIZE = 'aztec.circuit.simulation.output_size';

export const CIRCUIT_WITNESS_GEN_DURATION = 'aztec.circuit.witness_generation.duration';
export const CIRCUIT_WITNESS_GEN_INPUT_SIZE = 'aztec.circuit.witness_generation.input_size';
export const CIRCUIT_WITNESS_GEN_OUTPUT_SIZE = 'aztec.circuit.witness_generation.output_size';

export const CIRCUIT_PROVING_DURATION = 'aztec.circuit.proving.duration';
export const CIRCUIT_PROVING_INPUT_SIZE = 'aztec.circuit.proving.input_size';
export const CIRCUIT_PROVING_PROOF_SIZE = 'aztec.circuit.proving.proof_size';

export const CIRCUIT_PUBLIC_INPUTS_COUNT = 'aztec.circuit.public_inputs_count';
export const CIRCUIT_GATE_COUNT = 'aztec.circuit.gate_count';
export const CIRCUIT_SIZE = 'aztec.circuit.size';

export const MEMPOOL_TX_COUNT = 'aztec.mempool.tx_count';
export const MEMPOOL_TX_SIZE = 'aztec.mempool.tx_size';

export const ARCHIVER_SYNC_DURATION = 'aztec.archiver.sync_duration';
export const ARCHIVER_BLOCK_HEIGHT = 'aztec.archiver.block_height';
export const ARCHIVER_BLOCK_SIZE = 'aztec.archiver.block_size';

export const NODE_RECEIVE_TX_DURATION = 'aztec.node.receive_tx.duration';
export const NODE_RECEIVE_TX_COUNT = 'aztec.node.receive_tx.count';

export const SEQUENCER_BLOCK_BUILD_DURATION = 'aztec.sequencer.block.build_duration';
export const SEQUENCER_BLOCK_COUNT = 'aztec.sequencer.block.count';
export const SEQUENCER_CURRENT_STATE = 'aztec.sequencer.current.state';
export const SEQUENCER_CURRENT_BLOCK_NUMBER = 'aztec.sequencer.current.block_number';
export const SEQUENCER_CURRENT_BLOCK_SIZE = 'aztec.sequencer.current.block_size';

export const L1_PUBLISHER_GAS_PRICE = 'aztec.l1_publisher.gas_price';
export const L1_PUBLISHER_TX_COUNT = 'aztec.l1_publisher.tx_count';
export const L1_PUBLISHER_TX_DURATION = 'aztec.l1_publisher.tx_duration';
export const L1_PUBLISHER_TX_GAS = 'aztec.l1_publisher.tx_gas';
export const L1_PUBLISHER_TX_CALLDATA_SIZE = 'aztec.l1_publisher.tx_calldata_size';
export const L1_PUBLISHER_TX_CALLDATA_GAS = 'aztec.l1_publisher.tx_calldata_gas';

export const PUBLIC_PROCESSOR_TX_DURATION = 'aztec.public_processor.tx_duration';
export const PUBLIC_PROCESSOR_TX_COUNT = 'aztec.public_processor.tx_count';
export const PUBLIC_PROCESSOR_TX_PHASE_COUNT = 'aztec.public_processor.tx_phase_count';
export const PUBLIC_PROCESSOR_PHASE_DURATION = 'aztec.public_processor.phase_duration';
export const PUBLIC_PROCESSOR_PHASE_COUNT = 'aztec.public_processor.phase_count';
export const PUBLIC_PROCESSOR_DEPLOY_BYTECODE_SIZE = 'aztec.public_processor.deploy_bytecode_size';

export const PUBLIC_EXECUTOR_SIMULATION_COUNT = 'aztec.public_executor.simulation_count';
export const PUBLIC_EXECUTOR_SIMULATION_DURATION = 'aztec.public_executor.simulation_duration';
export const PUBLIC_EXECUTION_SIMULATION_BYTECODE_SIZE = 'aztec.public_executor.simulation_bytecode_size';

export const PROVING_ORCHESTRATOR_BASE_ROLLUP_INPUTS_DURATION =
  'aztec.proving_orchestrator.base_rollup.inputs_duration';

export const PROVING_QUEUE_JOB_SIZE = 'aztec.proving_queue.job_size';
export const PROVING_QUEUE_SIZE = 'aztec.proving_queue.size';
