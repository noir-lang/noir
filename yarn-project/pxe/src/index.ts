export * from './pxe_service/index.js';
export * from './pxe_http/index.js';
export * from './config/index.js';

export { Tx, TxHash } from '@aztec/circuit-types';

export { TxRequest, PartialAddress } from '@aztec/circuits.js';
export * from '@aztec/foundation/fields';
export * from '@aztec/foundation/eth-address';
export * from '@aztec/foundation/aztec-address';
export * from '@aztec/key-store';

// Temporarily used in e2e client prover integration test
export { BBNativeProofCreator } from './kernel_prover/bb_prover/bb_native_proof_creator.js';
