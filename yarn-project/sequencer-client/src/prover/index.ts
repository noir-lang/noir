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

export type Proof = UInt8Vector;
export interface RollupProver {
  getBaseRollupProof(input: BaseRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;
  getMergeRollupProof(input: MergeRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;
  getRootRollupProof(input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<Proof>;
}

export interface PublicProver {
  getPublicCircuitProof(publicInputs: PublicCircuitPublicInputs): Promise<Proof>;
  getPublicKernelCircuitProof(publicInputs: PublicKernelPublicInputs): Promise<Proof>;
}
