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

export const ARCHIVER_BLOCK_HEIGHT = 'aztec.archiver.block_height';
export const ARCHIVER_BLOCK_SIZE = 'aztec.archiver.block_size';
