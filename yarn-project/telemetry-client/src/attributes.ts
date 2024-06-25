/**
 * @overview This file contains the custom attributes used in telemetry events.
 * Attribute names exist in a global namespace, alongside metric names. Use this file to ensure that attribute names are unique.
 *
 * To define a new attribute follow these steps:
 * 1. Make sure it's not a semantic attribute that's already been defined by {@link @opentelemetry/semantic-conventions | OpenTelemetry} (e.g. `service.name`)
 * 2. Come up with a unique name for it so that it doesn't clash with other attributes or metrics.
 * 3. Prefix the attribute name with `aztec` to make it clear that it's a custom attribute.
 * 4. Add a description of what the attribute represents and examples of what it might contain.
 * 5. Start using it.
 *
 * @note Attributes and metric names exist in a hierarchy of namespaces. If a name has been used as a namespace, then it can not be used as a name for an attribute or metric.
 * @example If `aztec.circuit.name` has been defined as an attribute then `aztec.circuit` alone can not be re-used for a metric or attribute because it is already a namespace.
 * @see {@link https://opentelemetry.io/docs/specs/semconv/general/attribute-naming/}
 */

/**
 * The name of the protocol circuit being run (e.g. public-kernel-setup or base-rollup)
 * @see {@link @aztec/circuit-types/stats:CircuitName}
 */
export const PROTOCOL_CIRCUIT_NAME = 'aztec.circuit.protocol_circuit_name';

/**
 * The type of protocol circuit being run: server or client
 */
export const PROTOCOL_CIRCUIT_TYPE = 'aztec.circuit.protocol_circuit_type';

/**
 * For an app circuit, the contract:function being run (e.g. Token:transfer)
 */
export const APP_CIRCUIT_NAME = 'aztec.circuit.app_circuit_name';

/**
 * The type of app circuit being run: server or client
 */
export const APP_CIRCUIT_TYPE = 'aztec.circuit.app_circuit_type';

/** The block number */
export const BLOCK_NUMBER = 'aztec.block.number';
/** The parent's block number */
export const BLOCK_PARENT = 'aztec.block.parent';
/** How many txs are being processed to build this block */
export const BLOCK_CANDIDATE_TXS_COUNT = 'aztec.block.candidate_txs_count';
/** How many actual txs were included in this block */
export const BLOCK_TXS_COUNT = 'aztec.block.txs_count';
/** The block size (power of 2) */
export const BLOCK_SIZE = 'aztec.block.size';
/** The tx hash */
export const TX_HASH = 'aztec.tx.hash';
