import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  PublicCircuitPublicInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';

/**
 * Type definition for a circuit proof.
 */
export type Proof = UInt8Vector;

/**
 * Generates proofs for the base, merge, and root rollup circuits.
 */
export interface RollupProver {
  getBaseRollupProof(input: BaseRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;
  getMergeRollupProof(input: MergeRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;
  getRootRollupProof(input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<Proof>;
}

/**
 * Generates proofs for the public and public kernel circuits.
 */
export interface PublicProver {
  getPublicCircuitProof(publicInputs: PublicCircuitPublicInputs): Promise<Proof>;
  getPublicKernelCircuitProof(publicInputs: PublicKernelPublicInputs): Promise<Proof>;
}
