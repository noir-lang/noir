import { VerificationKey } from '@aztec/circuits.js';
import { makeVerificationKey } from '@aztec/circuits.js/factories';

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
 * Returns verification keys for each well known circuit.
 * TODO: Actually fetch real values.
 * @returns a VerificationKeys object.
 */
export function getVerificationKeys(): VerificationKeys {
  return {
    kernelCircuit: makeVerificationKey(),
    baseRollupCircuit: makeVerificationKey(),
  };
}
