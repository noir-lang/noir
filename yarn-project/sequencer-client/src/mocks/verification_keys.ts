import { VerificationKey } from '@aztec/circuits.js';

/**
 * Well-known verification keys.
 */
export interface VerificationKeys {
  /**
   * Verification key for the default private kernel circuit.
   */
  kernelCircuit: VerificationKey;
  /**
   * Verification key for the default base rollup circuit.
   */
  baseRollupCircuit: VerificationKey;
  /**
   * Verification key for the default merge rollup circuit.
   */
  mergeRollupCircuit: VerificationKey;
}

/**
 * Returns mock verification keys for each well known circuit.
 * @returns A VerificationKeys object with fake values.
 */
export function getVerificationKeys(): VerificationKeys {
  return {
    kernelCircuit: VerificationKey.makeFake(),
    baseRollupCircuit: VerificationKey.makeFake(),
    mergeRollupCircuit: VerificationKey.makeFake(),
  };
}
