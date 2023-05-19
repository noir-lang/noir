import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  Proof,
  PublicCircuitPublicInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';

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
