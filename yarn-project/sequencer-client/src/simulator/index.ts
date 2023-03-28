import {
  BaseRollupInputs,
  BaseRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';

export interface Simulator {
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseRollupPublicInputs>;
  mergeRollupCircuit(input: MergeRollupInputs): Promise<MergeRollupPublicInputs>;
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs>;
}
