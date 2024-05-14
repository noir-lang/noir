export { ProverClient } from '@aztec/circuit-types';

export * from './tx-prover/tx-prover.js';
export * from './config.js';
export * from './dummy-prover.js';

// Exported for integration_l1_publisher.test.ts
export { getVerificationKeys } from './mocks/verification_keys.js';
