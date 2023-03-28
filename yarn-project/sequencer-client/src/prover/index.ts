import {
  BaseRollupInputs,
  BaseRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';

export type Proof = UInt8Vector;
export interface Prover {
  getBaseRollupProof(input: BaseRollupInputs, publicInputs: BaseRollupPublicInputs): Promise<Proof>;
  getMergeRollupProof(input: MergeRollupInputs, publicInputs: MergeRollupPublicInputs): Promise<Proof>;
  getRootRollupProof(input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<Proof>;
}
