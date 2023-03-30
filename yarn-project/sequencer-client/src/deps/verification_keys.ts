import { VerificationKey } from '@aztec/circuits.js';

/**
 * Well-known verification keys
 */
export interface VerificationKeys {
  /**
   * Verification key for the default private kernel circuit
   */
  kernelCircuit: VerificationKey;
  /**
   * Verification key for the default base rollup circuit
   */
  baseRollupCircuit: VerificationKey;
}

/**
 * Returns mock verification keys for each well known circuit.
 * @returns a VerificationKeys object.
 */
export function getVerificationKeys(): VerificationKeys {
  return {
    kernelCircuit: VerificationKey.makeFake(),
    baseRollupCircuit: VerificationKey.makeFake(),
  };
}
