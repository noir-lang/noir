import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  PublicCircuitPublicInputs,
  PublicKernelInputsNoKernelInput,
  PublicKernelInputsNonFirstIteration,
  PublicKernelInputsPrivateKernelInput,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  TxRequest,
} from '@aztec/circuits.js';

export interface RollupSimulator {
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs>;
}

export interface PublicCircuitSimulator {
  publicCircuit(tx: TxRequest): Promise<PublicCircuitPublicInputs>;
}

export interface PublicKernelCircuitSimulator {
  publicKernelCircuitNoInput(inputs: PublicKernelInputsNoKernelInput): Promise<PublicKernelPublicInputs>;
  publicKernelCircuitPrivateInput(inputs: PublicKernelInputsPrivateKernelInput): Promise<PublicKernelPublicInputs>;
  publicKernelCircuitNonFirstIteration(inputs: PublicKernelInputsNonFirstIteration): Promise<PublicKernelPublicInputs>;
}
